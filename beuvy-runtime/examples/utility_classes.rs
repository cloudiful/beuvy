use beuvy_runtime::parse_utility_classes;

fn main() {
    let patch = parse_utility_classes("flex flex-col gap-[12px] px-[10px] py-[8px]")
        .expect("utility classes should parse");

    println!("{patch:#?}");
}
