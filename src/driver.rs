extern crate argparse;
extern crate typed_arena;

#[macro_use]
mod macros;

mod parse;
mod ty;

use std::fs::File;

// TODO(ubsan): unify parameter ordering
enum Either<L, R> {
  Left(L),
  Right(R),
}

// NOTE(ubsan): this should *not* be public
pub(crate) enum DebugPrint {
  Print,
  NoPrint,
}

impl From<bool> for DebugPrint {
  fn from(x: bool) -> DebugPrint {
    if x { DebugPrint::Print } else { DebugPrint::NoPrint }
  }
}

fn main() {
  use std::io::Read;

  let mut name = "".to_owned();
  let mut print_ast = false;
  let mut print_mir = false;
  {
    use argparse::{ArgumentParser, Store, StoreTrue};

    let mut ap = ArgumentParser::new();
    ap.set_description("\
      The pinkc compiler for the pink language.\n\
      Written in Rust.\
    "); ap.refer(&mut name).required().add_argument(
      "name", Store, "The file to compile"
    );
    ap.refer(&mut print_ast).add_option(
      &["--print-ast"], StoreTrue,
      "Pass if you would like to print the generated AST",
    );
    ap.refer(&mut print_mir).add_option(
      &["--print-mir"], StoreTrue,
      "Pass if you would like to print the generated MIR",
    );

    ap.parse_args_or_exit();
  }

  let print_ast = DebugPrint::from(print_ast);
  let print_mir = DebugPrint::from(print_mir);

  //let output = output.unwrap_or(get_output_from_name(&name));

  let mut file = Vec::new();
  File::open(&name).expect(&name).read_to_end(&mut file).unwrap();
  let file = String::from_utf8(file).unwrap();

  /*
  let tyctxt = ty::TypeContext::new();
  let ast = Ast::create(&str, &tyctxt);
  */
}
