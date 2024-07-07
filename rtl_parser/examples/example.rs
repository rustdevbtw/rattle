use rtl_parser::{parse, RtlResult};

fn main() -> RtlResult<()> {
    let r = parse(
        r#"
        import ::std as hi;
        f Add(Int x, Int y) Int
        struct Person {
            String name,
            Int age,
        }

        def Person {
            f From(String raw) This;
            f From(String name, Int age) This;
            f Greet(This this) String;
        } for SuperHuman;
    "#,
    )?;
    println!("{:#?}", r);
    Ok(())
}
