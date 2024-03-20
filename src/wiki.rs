//! This module is not used as of now, I found this info and noted this here in case it's useful in
//! the future.

// Reference:
// - https://en.wikipedia.org/wiki/Wikipedia:Administration#Data_structure_and_development
// - https://en.wikipedia.org/wiki/Wikipedia:Namespace
// - (Aliases, pseudo-namespaces) https://en.wikipedia.org/wiki/Wikipedia:Shortcut#List_of_prefixes
pub const NAMESPACES: [&'static str; 25] = [
    // Subject namespaces
    // "(Main/Article)" // This namespace is implicit and omitted in the URL.
    "Talk",
    //
    "User",
    "User talk",
    "Wikipedia",
    "Wikipedia talk",
    "File",
    "File talk",
    "MediaWiki",
    "MediaWiki talk",
    "Template",
    "Template talk",
    "Help",
    "Help talk",
    "Category",
    "Category talk",
    "Portal",
    "Portal talk",
    "Draft",
    "Draft talk",
    "TimedText",
    "TimedText talk",
    "Module",
    "Module talk",
    // Virtual namespaces
    "Special",
    "Media",
];

pub const NAMESPACE_ALIASES: [&str; 6] = [
    // Namespace aliases
    "WP",
    "WT",
    "Project",
    "Project talk",
    "Image",
    "Image talk",
];

pub const PSEUDO_NAMESPACES: [&str; 4] = [
    // Pseudo-namespaces
    "CAT", "H", "MOS", "P",
];
