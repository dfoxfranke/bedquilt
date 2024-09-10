use std::fmt::Display;
use walrus::{Export, Import, ValType};

#[derive(Debug)]
pub enum CompilationError {
    ValidationError(anyhow::Error),
    UnrecognizedImport(Import),
    IncorrectlyTypedImport {
        import: Import,
        expected: (Vec<ValType>, Vec<ValType>),
        actual: (Vec<ValType>, Vec<ValType>),
    },
    IncorrectlyTypedExport {
        export: Export,
        expected: (Vec<ValType>, Vec<ValType>),
        actual: (Vec<ValType>, Vec<ValType>),
    },
    NoEntrypoint,
    Overflow(OverflowLocation),
    UnsupportedMultipleMemories,
    UnsupportedInstruction {
        function: Option<String>,
        instr: &'static str,
    },
    InputError(std::io::Error),
    OutputError(std::io::Error),
    OtherError(anyhow::Error),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum OverflowLocation {
    /// A type declares too many parameters or return values
    TypeDecl,
    /// Too many types declared
    TypeList,
    /// Too many functions
    FnList,
    /// Too many local variables in named function
    Locals(Option<String>),
    /// Table too large
    Table,
    /// Element segment too large
    Element,
    /// Data segment too large
    Data,
    /// Initial memory size too large
    Memory,
    /// Final assembled output too large
    FinalAssembly,
}

impl Display for CompilationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CompilationError::ValidationError(e) => write!(f, "Module validation error: {}", e)?,
            CompilationError::UnrecognizedImport(import) => {
                match import.kind {
                    walrus::ImportKind::Function(_) => write!(f, "Unrecognized function import: ")?,
                    walrus::ImportKind::Table(_) => write!(f, "Unrecognized table import: ")?,
                    walrus::ImportKind::Memory(_) => write!(f, "Unrecognized memory import: ")?,
                    walrus::ImportKind::Global(_) => write!(f, "Unrecognized global import: ")?,
                }

                write!(f, "{}/{}", import.module, import.name)?;
                if import.module == "env" {
                    write!(
                        f,
                        " (Did you mean to specify a module name to override the default \"env\"?)"
                    )?;
                }
            }
            CompilationError::IncorrectlyTypedImport {
                import,
                expected,
                actual,
            } => {
                let expected_params: Vec<String> =
                    expected.0.iter().map(|vt| vt.to_string()).collect();
                let expected_results: Vec<String> =
                    expected.1.iter().map(|vt| vt.to_string()).collect();
                let actual_params: Vec<String> = actual.0.iter().map(|vt| vt.to_string()).collect();
                let actual_results: Vec<String> =
                    actual.1.iter().map(|vt| vt.to_string()).collect();

                write!(
                    f,
                    "Incorrectly-typed import of {}/{}.\n    Expected: ({}) -> ({})\n    Actual:   ({}) -> ({})", 
                    import.module,
                    import.name,
                    expected_params.join(","),
                    expected_results.join(","),
                    actual_params.join(","),
                    actual_results.join(","),
                )?;
            }
            CompilationError::IncorrectlyTypedExport {
                export,
                expected,
                actual,
            } => {
                let expected_params: Vec<String> =
                    expected.0.iter().map(|vt| vt.to_string()).collect();
                let expected_results: Vec<String> =
                    expected.1.iter().map(|vt| vt.to_string()).collect();
                let actual_params: Vec<String> = actual.0.iter().map(|vt| vt.to_string()).collect();
                let actual_results: Vec<String> =
                    actual.1.iter().map(|vt| vt.to_string()).collect();

                write!(
                    f,
                    "Incorrectly-typed export of {}.\n    Expected: ({}) -> ({})\n    Actual:   ({}) -> ({})",
                    export.name,
                    expected_params.join(","),
                    expected_results.join(","),
                    actual_params.join(","),
                    actual_results.join(","),
                )?;
            }
            CompilationError::NoEntrypoint => {
                write!(f, "Module contains no entrypoint. Provide a start function or export a function named glulx_main.")?;
            }
            CompilationError::Overflow(loc) => {
                match loc {
                    OverflowLocation::TypeDecl => write!(f, "A type declaration ")?,
                    OverflowLocation::TypeList => write!(f, "The module's list of types ")?,
                    OverflowLocation::FnList => write!(f, "The module's list of functions ")?,
                    OverflowLocation::Locals(None) => {
                        write!(f, "The set of local variables used by an unnamed function ")?
                    }
                    OverflowLocation::Locals(Some(name)) => write!(
                        f,
                        "The set of local variables used by the function `{}` ",
                        name
                    )?,
                    OverflowLocation::Table => write!(f, "A table declaration ")?,
                    OverflowLocation::Element => write!(f, "An element segment ")?,
                    OverflowLocation::Data => write!(f, "A data segment ")?,
                    OverflowLocation::Memory => write!(f, "The program memory ")?,
                    OverflowLocation::FinalAssembly => write!(f, "The assembled output ")?,
                }
                write!(f, "overflows Glulx's 4GiB address space")?;
            }
            CompilationError::UnsupportedMultipleMemories => {
                write!(f, "Modules that define multiple memories are not supported")?;
            }
            CompilationError::UnsupportedInstruction { function, instr } => {
                if let Some(function) = function {
                    write!(
                        f,
                        "Encountered an unsupported instruction in function {}: {:?}",
                        function, instr
                    )?
                } else {
                    write!(
                        f,
                        "Encountered an unsupported instruction in an unnamed function: {:?}",
                        instr
                    )?
                }
            }
            CompilationError::InputError(e) => {
                write!(f, "While reading input: {}", e)?;
            }
            CompilationError::OutputError(e) => {
                write!(f, "While writing output: {}", e)?;
            }
            CompilationError::OtherError(e) => {
                write!(f, "{}", e)?;
            }
        }
        Ok(())
    }
}

impl std::error::Error for CompilationError {}
