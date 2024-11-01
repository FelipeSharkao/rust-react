use rust_react_template::template;

fn main() {
    let t = template! {
        <span>Test {a} [a] (a)</span>
    };
    dbg!(t);
}
