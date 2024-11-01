use rust_react::ReactElement;
use rust_react_template::template;

fn main() {
    let t = template! {
        <Main>Test</Main>
    };
    dbg!(t);
}

struct MainProps {}
#[allow(non_snake_case)]
fn Main(props: MainProps) -> ReactElement {
    template! {
        <span>Test {a} [a] (a)</span>
    }
}
