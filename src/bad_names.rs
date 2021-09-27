use rand::random;

const ADJECTIVES: &[&str] = &[
    "German",
    "Dungeon",
    "Naughty",
    "Explicit",
    "Uncensored",
    "Dirty",
    "Family-friendly",
    "NSFW",
    "\"legal\"",
];

const NOUNS: &[&str] = &[
    "Daddy",
    "Video",
    "Collage",
    "School-Assignment",
    "ResearchPaper",
    "Step-brothers",
    "Party",
];

const FILE_EXTENSIONS: &[&str] = &[".txt", ".pdf", ".xxx", ".mov", ".torrent", ".cpp", ".bing"];

enum SubPattern {
    Str(&'static str),
    Noun,
    Adj,
    Ext,
}

pub fn generate() -> String {
    use SubPattern::*;
    const VARIANTS: &[&[SubPattern]] = &[
        &[Adj, Str("-"), Noun, Str("("), Adj, Str(")"), Ext],
        &[Adj, Str("_"), Noun],
        &[Adj, Str("_"), Noun, Ext],
        &[Adj, Str("-"), Noun, Ext],
        &[Adj, Adj, Noun, Ext],
        &[Str("/"), Adj, Str("/"), Adj, Str("/"), Adj, Ext],
    ];

    let random_arr = |arr: &[&'static str]| arr[random::<usize>() % arr.len()];

    let part_to_str = |part: &SubPattern| match part {
        Str(s) => s,
        Adj => random_arr(ADJECTIVES),
        Noun => random_arr(NOUNS),
        Ext => random_arr(FILE_EXTENSIONS),
    };

    let variant = random::<usize>() % VARIANTS.len();

    VARIANTS[variant]
        .iter()
        .map(part_to_str)
        .fold(String::new(), |mut bucket, part| {
            bucket.push_str(part);
            bucket
        })
}
