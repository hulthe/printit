use crate::PrintOpt;

pub fn build_cmd(printer: &str, opts: &PrintOpt) -> String {
    format!(
        "{bin} -P '{printer}' -# {copies} {color_mode} {media} {one_sided} {ranges}",
        bin = "/usr/bin/lpr",
        printer = printer,
        copies = opts.copies,
        color_mode = match (opts.color, opts.grayscale) {
            // NOTE: ColorModel RGB might be non-standard
            // run "lpoptions -p ed-6229-color3 -l" on the remote to find out
            // available color modes of (for example) printer ed-6229-color3
            (true, false) => "-o 'ColorModel=RGB' -o 'ColorMode=True'",
            (false, true) => "-o 'ColorModel=Gray' -o 'ColorMode=False'",
            (false, false) => "",
            (true, true) => unreachable!("conflicting color modes"),
        },
        media = match &opts.media {
            Some(size) => format!("-o media='{}'", size),
            None => format!(""),
        },
        one_sided = match opts.one_sided {
            true => "-o sides=one-sided",
            false => "-o sides=two-sided-long-edge",
        },
        ranges = opts
            .range
            .iter()
            .map(|r| {
                if r.start() == r.end() {
                    format!("{}", r.start())
                } else {
                    format!("{}-{}", r.start(), r.end())
                }
            })
            .reduce(|a, b| [a, b].join(","))
            .map(|ranges| format!("-o page-ranges={}", ranges))
            .unwrap_or_default(),
    )
}
