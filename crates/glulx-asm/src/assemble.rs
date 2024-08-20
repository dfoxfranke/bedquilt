// SPDX-License-Identifier: Apache-2.0 WITH LLVM-Exception
// Copyright 2024 Daniel Fox Franke.

//! Main assembler implementation.

use alloc::borrow::{Borrow, Cow};
use bytes::{Buf, BufMut, Bytes, BytesMut};
use core::{hash::Hash, ops::Range};

#[cfg(not(feature = "std"))]
use hashbrown::HashMap;
#[cfg(feature = "std")]
use std::collections::HashMap;

use crate::{
    cast::{checked_next_multiple_of, Overflow},
    error::AssemblerError,
    items::{Item, LabelRef, ZeroItem},
    resolver::{ResolvedAddr, Resolver},
};

/// Length of the story file header.
const HEADER_LENGTH: u32 = 0x24;
/// Magic number identifying a Glulx story file.
const MAGIC_NUMBER: u32 = 0x476C756C;
/// The Glulx version we're implementing (3.1.3).
const GLULX_VERSION: u32 = 0x00030103;

/// Resolve labels from a hash table lookup.
struct HashResolver<'a, L> {
    hashmap: &'a HashMap<L, Range<u32>>,
    ramstart: u32,
}

impl<L> Resolver for HashResolver<'_, L>
where
    L: Clone + Hash + Eq,
{
    type Label = L;

    fn resolve(&self, label: &Self::Label) -> Result<ResolvedAddr, AssemblerError<Self::Label>> {
        let addr = self
            .hashmap
            .get(label)
            .ok_or_else(|| AssemblerError::UndefinedLabel(label.clone()))?
            .start;
        if addr < self.ramstart {
            Ok(ResolvedAddr::Rom(addr))
        } else {
            Ok(ResolvedAddr::Ram(addr - self.ramstart))
        }
    }
}

/// Collection of all inputs needed to assemble a story file.
#[derive(Debug, Clone)]
pub struct Assembly<'a, L>
where
    L: Clone,
{
    /// List of items which should appear in the ROM section.
    pub rom_items: Cow<'a, [(Option<L>, Item<L>)]>,
    /// List of items which should appear in the RAM section.
    pub ram_items: Cow<'a, [(Option<L>, Item<L>)]>,
    /// List of items which should have space in the RAM section and be initialized to zero.
    pub zero_items: Cow<'a, [(Option<L>, ZeroItem)]>,
    /// How much space to allocate for the stack.
    pub stack_size: u32,
    /// Reference to the function to be called at the start of execution.
    pub start_func: LabelRef<L>,
    /// Reference to the initial decoding table.
    pub decoding_table: Option<LabelRef<L>>,
}

impl<L> Assembly<'_, L>
where
    L: Clone + Eq + Hash,
{
    /// Applies the given mapping function to all labels within the assembly.
    pub fn map<F, M>(self, mut f: F) -> Assembly<'static, M>
    where
        F: FnMut(L) -> M,
        M: Clone,
    {
        let rom_items = self
            .rom_items
            .iter()
            .cloned()
            .map(|(label, item)| (label.map(&mut f), item.map(&mut f)))
            .collect();

        let ram_items = self
            .ram_items
            .iter()
            .cloned()
            .map(|(label, item)| (label.map(&mut f), item.map(&mut f)))
            .collect();

        let zero_items = self
            .zero_items
            .iter()
            .cloned()
            .map(|(label, item)| (label.map(&mut f), item))
            .collect();

        let stack_size = self.stack_size;

        let start_func = self.start_func.map(&mut f);
        let decoding_table = self.decoding_table.map(|r| r.map(&mut f));

        Assembly {
            rom_items,
            ram_items,
            zero_items,
            stack_size,
            start_func,
            decoding_table,
        }
    }

    /// Assembles a Glulx binary, ready to be written out as a `.ulx` file.
    pub fn assemble(&self) -> Result<BytesMut, AssemblerError<L>> {
        assemble(
            self.rom_items.borrow(),
            self.ram_items.borrow(),
            self.zero_items.borrow(),
            self.stack_size,
            &self.start_func,
            &self.decoding_table,
        )
    }

    /// Converts all internal [`Cow`] fields to owned.
    pub fn to_owning(&self) -> Assembly<'static, L> {
        Assembly {
            rom_items: Cow::Owned(self.rom_items.clone().into_owned()),
            ram_items: Cow::Owned(self.ram_items.clone().into_owned()),
            zero_items: Cow::Owned(self.zero_items.clone().into_owned()),
            stack_size: self.stack_size,
            start_func: self.start_func.clone(),
            decoding_table: self.decoding_table.clone(),
        }
    }
}

/// Top-level function of our main assembler algorithm.
///
/// The hard part of this is dealing with variable-length operands, and
/// especially dealing with the PC-relative offset operands used by branch
/// instructions. The approach is basically:
///
/// 1. Start by computing label positions based on the worst case in which every
///    label-based operand requires full-width encoding.
///
/// 2. Compute encoding lengths based on everything being at the positions we
///    computed in step 1, and then compute new positions based on those
///    lengths.
///
/// 3. Repeat step 2 based on the results from the previous iteration, and keep
///    iterating until we get to a fixed point.
///
///    Operands should only ever shrink, lengths are natural numbers, and the
///    natural numbers are well-ordered, so the Tarski fixed-point theorem
///    should guarantee termination. Making sure of "operands should only ever
///    shrink" is a little tricky, because we have to be careful that, as a
///    result of shrinking a branch-offset operand, it gets further away from
///    the thing it's branching to and so it needs to grow again to encode the
///    larger offset. But that trickiness is handled in
///    [`LoadOperand::resolve`], not in this function.
///
/// 4. Finally, serialize the output, checking assertions along the way to make
///    sure the lengths we got are the ones we planned to get.
fn assemble<L>(
    rom_items: &[(Option<L>, Item<L>)],
    ram_items: &[(Option<L>, Item<L>)],
    zero_items: &[(Option<L>, ZeroItem)],
    stack_size: u32,
    start_func: &LabelRef<L>,
    decoding_table: &Option<LabelRef<L>>,
) -> Result<BytesMut, AssemblerError<L>>
where
    L: Clone + Eq + Hash,
{
    let mut unlabel_ctr: usize = 0;
    let mut labeled: HashMap<L, Range<u32>> = HashMap::new();
    let mut unlabeled: HashMap<usize, Range<u32>> = HashMap::new();

    let mut position = HEADER_LENGTH;

    // Step 1: initialize positions
    initialize_positions(
        rom_items,
        &mut labeled,
        &mut unlabeled,
        &mut position,
        &mut unlabel_ctr,
    )?;
    position = checked_next_multiple_of(position, 256)?;
    let mut ramstart = position;
    initialize_positions(
        ram_items,
        &mut labeled,
        &mut unlabeled,
        &mut position,
        &mut unlabel_ctr,
    )?;
    position = checked_next_multiple_of(position, 256)?;
    initialize_zero_positions(
        zero_items,
        &mut labeled,
        &mut unlabeled,
        &mut position,
        &mut unlabel_ctr,
    )?;

    // Step 2/3: update positions until we reach a fixed point.
    loop {
        position = HEADER_LENGTH;
        unlabel_ctr = 0;

        let rom_improved = update_positions(
            rom_items,
            &mut labeled,
            &mut unlabeled,
            &mut position,
            &mut unlabel_ctr,
            ramstart,
        )?;
        position = checked_next_multiple_of(position, 256)?;
        ramstart = position;
        let ram_improved = update_positions(
            ram_items,
            &mut labeled,
            &mut unlabeled,
            &mut position,
            &mut unlabel_ctr,
            ramstart,
        )?;
        position = checked_next_multiple_of(position, 256)?;
        let zero_improved = update_zero_positions(
            zero_items,
            &mut labeled,
            &mut unlabeled,
            &mut position,
            &mut unlabel_ctr,
        )?;

        if !rom_improved && !ram_improved && !zero_improved {
            break;
        }
    }

    // Step 4: serialize output.
    let mut body = BytesMut::new();
    serialize_items(rom_items, &labeled, ramstart, &mut body)?;
    assert_eq!(
        ramstart
            .checked_sub(HEADER_LENGTH)
            .expect("ramstart should be >= HEADER_LENGTH"),
        body.len().try_into().overflow()?
    );
    serialize_items(ram_items, &labeled, ramstart, &mut body)?;

    let body = body.freeze();
    let extstart = u32::try_from(body.len())
        .overflow()?
        .checked_add(HEADER_LENGTH)
        .overflow()?;

    let endmem = checked_next_multiple_of(verify_zero_items(zero_items, &labeled, extstart)?, 256)?;

    let resolver = HashResolver {
        hashmap: &labeled,
        ramstart,
    };

    let resolved_decoding_table = if let Some(decoding_table) = decoding_table {
        decoding_table.resolve_absolute(ramstart, &resolver)?
    } else {
        0u32
    };

    let resolved_start_func: u32 = start_func.resolve_absolute(ramstart, &resolver)?;

    let sum = MAGIC_NUMBER
        .wrapping_add(GLULX_VERSION)
        .wrapping_add(ramstart)
        .wrapping_add(extstart)
        .wrapping_add(endmem)
        .wrapping_add(stack_size)
        .wrapping_add(resolved_start_func)
        .wrapping_add(resolved_decoding_table)
        .wrapping_add(checksum(body.clone()));

    let mut output = BytesMut::with_capacity(
        body.len()
            .checked_add(
                usize::try_from(HEADER_LENGTH).expect("u32 to usize conversion should succeed"),
            )
            .overflow()?,
    );

    output.put_u32(MAGIC_NUMBER);
    output.put_u32(GLULX_VERSION);
    output.put_u32(ramstart);
    output.put_u32(extstart);
    output.put_u32(endmem);
    output.put_u32(stack_size);
    output.put_u32(resolved_start_func);
    output.put_u32(resolved_decoding_table);
    output.put_u32(sum);
    output.put(body);

    Ok(output)
}

/// Initializes item positions for the first step of assembly.
fn initialize_positions<L>(
    items: &[(Option<L>, Item<L>)],
    labeled: &mut HashMap<L, Range<u32>>,
    unlabeled: &mut HashMap<usize, Range<u32>>,
    position: &mut u32,
    unlabel_ctr: &mut usize,
) -> Result<(), AssemblerError<L>>
where
    L: Clone + Hash + Eq,
{
    for (label, item) in items {
        let worst_len: u32 = item.worst_len().try_into().overflow()?;
        let end_position = position.checked_add(worst_len).overflow()?;
        if let Some(label) = label {
            if labeled
                .insert(label.clone(), *position..end_position)
                .is_some()
            {
                return Err(AssemblerError::DuplicateLabel(label.clone()));
            }
        } else {
            assert!(unlabeled
                .insert(*unlabel_ctr, *position..end_position)
                .is_none());
            *unlabel_ctr += 1;
        }

        *position = checked_next_multiple_of(end_position, item.align())?;
    }

    Ok(())
}

/// Initializes zero-item positions for the first step of assembly.
fn initialize_zero_positions<L>(
    items: &[(Option<L>, ZeroItem)],
    labeled: &mut HashMap<L, Range<u32>>,
    unlabeled: &mut HashMap<usize, Range<u32>>,
    position: &mut u32,
    unlabel_ctr: &mut usize,
) -> Result<(), AssemblerError<L>>
where
    L: Clone + Hash + Eq,
{
    for (label, item) in items {
        let end_position = position.checked_add(item.len()).overflow()?;
        if let Some(label) = label {
            if labeled
                .insert(label.clone(), *position..end_position)
                .is_some()
            {
                return Err(AssemblerError::DuplicateLabel(label.clone()));
            };
        } else {
            assert!(unlabeled
                .insert(*unlabel_ctr, *position..end_position)
                .is_none());
            *unlabel_ctr += 1;
        }

        *position = checked_next_multiple_of(end_position, item.align())?;
    }

    Ok(())
}

/// Called from each iterative step to update item positions.
///
/// We're keeping track of start and end positions for both labeled and
/// unlabeled items, which is probably overkill; just tracking label positions
/// should be sufficient. But I don't want to try ripping out these extra checks
/// until I have a good test suite to confirm that doing so didn't break
/// anything.
fn update_positions<L>(
    items: &[(Option<L>, Item<L>)],
    labeled: &mut HashMap<L, Range<u32>>,
    unlabeled: &mut HashMap<usize, Range<u32>>,
    position: &mut u32,
    unlabel_ctr: &mut usize,
    ramstart: u32,
) -> Result<bool, AssemblerError<L>>
where
    L: Clone + Hash + Eq,
{
    let mut improvement_found = false;
    for (label, item) in items {
        let old_range = if let Some(label) = label {
            labeled
                .get(label)
                .expect("previously-inserted label should still be in the HashMap")
                .clone()
        } else {
            unlabeled
                .get(unlabel_ctr)
                .expect("previously-inserted label should still be in the HashMap")
                .clone()
        };

        let resolver = HashResolver {
            hashmap: labeled,
            ramstart,
        };

        let resolved_len = item.resolved_len(*position, ramstart, &resolver)?;

        let end_position = position
            .checked_add(u32::try_from(resolved_len).overflow()?)
            .overflow()?;

        if *position != old_range.start || end_position != old_range.end {
            improvement_found = true;
            if let Some(label) = label {
                labeled.insert(label.clone(), *position..end_position);
            } else {
                unlabeled.insert(*unlabel_ctr, *position..end_position);
            }
        }

        if label.is_none() {
            *unlabel_ctr += 1;
        }
        *position = checked_next_multiple_of(end_position, item.align())?;
    }
    Ok(improvement_found)
}

/// Called from each iterative step to update zero-item positions.
fn update_zero_positions<L>(
    items: &[(Option<L>, ZeroItem)],
    labeled: &mut HashMap<L, Range<u32>>,
    unlabeled: &mut HashMap<usize, Range<u32>>,
    position: &mut u32,
    unlabel_ctr: &mut usize,
) -> Result<bool, AssemblerError<L>>
where
    L: Clone + Hash + Eq,
{
    let mut improvement_found = false;

    for (label, item) in items {
        let old_range = if let Some(label) = label {
            labeled
                .get(label)
                .expect("previously-inserted label should still be in the HashMap")
                .clone()
        } else {
            unlabeled
                .get(unlabel_ctr)
                .expect("previously-inserted label should still be in the HashMap")
                .clone()
        };
        let end_position = position.checked_add(item.len()).overflow()?;
        if *position != old_range.start || end_position != old_range.end {
            improvement_found = true;
            if let Some(label) = label {
                labeled.insert(label.clone(), *position..end_position);
            } else {
                unlabeled.insert(*unlabel_ctr, *position..end_position);
            }
        }

        if label.is_none() {
            *unlabel_ctr += 1;
        }
        *position = checked_next_multiple_of(end_position, item.align())?;
    }

    Ok(improvement_found)
}

/// Serializes items after all final label positions have been computed.
fn serialize_items<L>(
    items: &[(Option<L>, Item<L>)],
    labeled: &HashMap<L, Range<u32>>,
    ramstart: u32,
    buf: &mut BytesMut,
) -> Result<(), AssemblerError<L>>
where
    L: Clone + Eq + Hash,
{
    for (label, item) in items {
        let position = u32::try_from(buf.len())
            .overflow()?
            .checked_add(HEADER_LENGTH)
            .overflow()?;

        if let Some(label) = label {
            let range = labeled
                .get(label)
                .ok_or_else(|| AssemblerError::UndefinedLabel(label.clone()))?;
            assert_eq!(
                range.start, position,
                "label position should match previous calculation"
            );
        }

        let resolver = HashResolver {
            hashmap: labeled,
            ramstart,
        };

        item.serialize(position, ramstart, &resolver, &mut *buf)?;
    }

    let position = u32::try_from(buf.len())
        .overflow()?
        .checked_add(HEADER_LENGTH)
        .overflow()?;

    let page_offset = position % 256;
    let padding = usize::try_from(if page_offset == 0 {
        0
    } else {
        256 - page_offset
    })
    .expect("u32 to usize conversion should succeed");
    buf.put_bytes(0, padding);

    Ok(())
}

/// Checks assertions to ensure that all zero-items were placed as intended.
fn verify_zero_items<L>(
    items: &[(Option<L>, ZeroItem)],
    labeled: &HashMap<L, Range<u32>>,
    extstart: u32,
) -> Result<u32, AssemblerError<L>>
where
    L: Clone + Eq + Hash,
{
    let mut position = extstart;

    for (label, item) in items {
        if let Some(label) = label {
            let range = labeled
                .get(label)
                .ok_or_else(|| AssemblerError::UndefinedLabel(label.clone()))?;
            assert_eq!(
                position, range.start,
                "label position should match previous calculation"
            );
        }

        position = position.checked_add(item.len()).overflow()?;
        position = checked_next_multiple_of(position, item.align())?;
    }

    Ok(position)
}

/// Header checksum calculation.
fn checksum(mut bytes: Bytes) -> u32 {
    let mut sum: u32 = 0;
    while bytes.has_remaining() {
        sum = sum.wrapping_add(bytes.get_u32());
    }
    sum
}
