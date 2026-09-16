pub mod blocks;
pub mod collections;
/// Parser modules for different AST components
///
/// This module organizes the parsing logic into separate, focused modules:
/// - `expressions`: Handles expression parsing (literals, identifiers, assignments)
/// - `items`: Handles item parsing (objects, methods, properties, etc.)
/// - `blocks`: Handles block and statement parsing
/// - `collections`: Handles array and set parsing
pub mod expressions;
pub mod items;
