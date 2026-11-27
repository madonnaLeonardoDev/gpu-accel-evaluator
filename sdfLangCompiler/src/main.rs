mod sflc;
use sflc::sfl_lexer::tokenize_sfl;
use sflc::sfl_parser::SdflParser;
fn main() {
    let test_input = r#"
        const radius = 10 + 2; 

        shape TestShape(x, y, z){
            x + y-10 + z
        }

        scene(x,y,z)[
        x + sin(y^2 * z) - radius;
        TestShape(x, 2, z-1);
        ]
    "#;

    let tokens = tokenize_sfl(test_input).exec_res();
    println!("I55 {:?}", tokens[55]);
    let mut parser = SdflParser::new(tokens);
    let res = parser.wgsl_parse().exec_res();
    println!("{}", res);
}
trait ExecResultsMain<R> {
    fn exec_res(self) -> R;
}
impl<R> ExecResultsMain<R> for Result<R, String> {
    fn exec_res(self) -> R {
        match self {
            Ok(r) => r,
            Err(e) => {
                eprint!("{}", e);
                std::process::exit(1);
            }
        }
    }
}
