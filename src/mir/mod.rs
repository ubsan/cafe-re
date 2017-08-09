// TODO(ubsan): make sure to start dealing with Spanneds
// whee errors are fun

mod runner;

use ast::{Ast, StringlyType};
use containers::ArenaMap;

use self::runner::Runner;

use std::fmt::{self, Display};

#[derive(Copy, Clone, Debug)]
pub enum IntSize {
  //I8,
  //I16,
  I32,
  //I64,
  // ISize,
  // I128,
}
impl IntSize {
  fn size(self) -> u32 {
    match self {
      IntSize::I32 => 32,
    }
  }
}
#[derive(Debug)]
pub enum BuiltinType {
  SInt(IntSize),
  //UInt(IntSize),
  //Bool,
}
impl Display for BuiltinType {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match *self {
      BuiltinType::SInt(size) => write!(f, "s{}", size.size()),
    }
  }
}

#[derive(Debug)]
pub enum TypeVariant<'tcx> {
  Builtin(BuiltinType),
  __LifetimeHolder(::std::marker::PhantomData<&'tcx ()>),
}
impl<'tcx> TypeVariant<'tcx> {
  pub fn s32() -> Self {
    TypeVariant::Builtin(BuiltinType::SInt(IntSize::I32))
  }
}
impl<'tcx> Display for TypeVariant<'tcx> {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match *self {
      TypeVariant::Builtin(ref builtin) => {
        write!(f, "{}", builtin)
      }
      TypeVariant::__LifetimeHolder(_) => {
        panic!()
      }
    }
  }
}

#[derive(Copy, Clone, Debug)]
pub struct Reference(u32);

impl Reference {
  pub fn ret() -> Self {
    Reference(0)
  }
}

#[derive(Copy, Clone, Debug)]
pub enum Value {
  Literal(i32),
  Reference(Reference),
  Add(Reference, Reference),
  Call {
    callee: FunctionDecl,
  }
}

#[derive(Copy, Clone, Debug)]
pub struct Block(u32);

#[derive(Debug)]
pub enum Terminator {
  Goto(Block),
  Return,
}

#[derive(Copy, Clone, Debug)]
struct Statement {
  lhs: Reference,
  rhs: Value,
}

#[derive(Debug)]
struct BlockData {
  num: Block,
  stmts: Vec<Statement>,
  term: Terminator,
}

#[derive(Debug)]
struct FunctionValue<'ctx> {
  ret_ty: Type<'ctx>,
  blks: Vec<BlockData>,
  locals: Vec<Type<'ctx>>,
  bindings: Vec<(Option<String>, Type<'ctx>)>,
}

#[derive(Debug)]
pub struct FunctionBuilder<'ctx> {
  decl: FunctionDecl,
  ret_ty: Type<'ctx>,
  locals: Vec<Type<'ctx>>,
  bindings: Vec<(Option<String>, Type<'ctx>)>,
  blks: Vec<BlockData>,
}

impl<'ctx> FunctionBuilder<'ctx> {
  fn new(decl: FunctionDecl, ret_ty: Type<'ctx>) -> Self {
    let enter_block = BlockData {
      num: Block(0),
      stmts: vec![],
      term: Terminator::Goto(Block(1)),
    };
    let exit_block = BlockData {
      num: Block(1),
      stmts: vec![],
      term: Terminator::Return,
    };
    FunctionBuilder {
      decl,
      ret_ty,
      locals: vec![],
      bindings: vec![(Some(String::from("<return>")), ret_ty)],
      blks: vec![enter_block, exit_block],
    }
  }

  pub fn entrance(&self) -> Block {
    Block(0)
  }
}

impl<'ctx> FunctionBuilder<'ctx> {
  pub fn add_stmt(
    &mut self,
    blk: Block,
    lhs: Reference,
    rhs: Value,
  ) {
    let blk_data = &mut self.blks[blk.0 as usize];
    blk_data.stmts.push(Statement { lhs, rhs });
  }

  pub fn add_anonymous_local(
    &mut self,
    ty: Type<'ctx>,
  ) -> Reference {
    self.locals.push(ty);
    self.bindings.push((None, ty));
    Reference((self.bindings.len() - 1) as u32)
  }

  pub fn add_local(
    &mut self,
    name: String,
    ty: Type<'ctx>,
  ) -> Reference {
    self.locals.push(ty);
    self.bindings.push((Some(name), ty));
    Reference((self.bindings.len() - 1) as u32)
  }
}

#[derive(Copy, Clone, Debug)]
pub struct Type<'ctx>(&'ctx TypeVariant<'ctx>);
#[derive(Copy, Clone, Debug)]
pub struct FunctionDecl(usize);

// NOTE(ubsan): when I get namespacing, I should probably
// use paths instead of names?

pub struct MirCtxt<'a> {
  types: ArenaMap<String, TypeVariant<'a>>,
}

impl<'a> MirCtxt<'a> {
  pub fn new() -> Self {
    MirCtxt {
      types: ArenaMap::new(),
    }
  }
}

pub struct Mir<'ctx> {
  funcs: Vec<(Option<String>, Option<FunctionValue<'ctx>>)>,
  types: &'ctx ArenaMap<String, TypeVariant<'ctx>>,
}

impl<'ctx> Mir<'ctx> {
  pub fn new(ctx: &'ctx MirCtxt<'ctx>, mut ast: Ast) -> Self {
    let mut self_: Mir<'ctx> = Mir {
      funcs: vec![],
      types: &ctx.types,
    };

    ast.build_mir(&mut self_);

    self_
  }

  pub fn run(&self) -> i32 {
    for (i, &(ref name, _)) in self.funcs.iter().enumerate() {
      if let Some("main") = name.as_ref().map(|s| &**s) {
        return Runner::new(self).call(FunctionDecl(i));
      }
    }
    panic!("no main function found")
  }
}

impl<'ctx> Mir<'ctx> {
  pub fn insert_type(
    &self,
    name: Option<String>,
    ty: TypeVariant<'ctx>,
  ) -> Type<'ctx> {
    if let Some(name) = name {
      Type(self.types.insert(name, ty))
    } else {
      Type(self.types.insert_anonymous(ty))
    }
  }

  pub fn add_function_decl(
    &mut self,
    name: Option<String>,
  ) -> FunctionDecl {
    self.funcs.push((name, None));
    FunctionDecl(self.funcs.len() - 1)
  }

  pub fn get_function_builder(
    &self,
    decl: FunctionDecl,
    ret_ty: Type<'ctx>,
  ) -> FunctionBuilder<'ctx> {
    FunctionBuilder::new(decl, ret_ty)
  }

  pub fn add_function_definition(
    &mut self,
    builder: FunctionBuilder<'ctx>,
  ) {
    let value = FunctionValue {
      ret_ty: builder.ret_ty,
      blks: builder.blks,
      locals: builder.locals,
      bindings: builder.bindings,
    };

    self.funcs[builder.decl.0].1 = Some(value);
  }

  pub fn get_type(
    &self,
    stype: &StringlyType,
  ) -> Option<Type<'ctx>> {
    match *stype {
      StringlyType::UserDefinedType(ref name) => {
        self.types.get(name).map(|t| Type(t))
      }
      _ => unimplemented!(),
    }
  }
}

impl<'ctx> Mir<'ctx> {
  pub fn print(&self) {
    fn binding_name(binding: &Option<String>) -> &str {
      binding.as_ref().map(|s| &**s).unwrap_or("")
    }
    fn print_binding(
      bindings: &[(Option<String>, Type)],
      r: Reference,
    ) {
      let name = binding_name(&bindings[r.0 as usize].0);
      print!("{}_{}", name, r.0);
    }

    for (name, ty) in &*self.types.hashmap() {
      print!("type {} :: ", name);
      match *unsafe { &**ty } {
        TypeVariant::Builtin(_) => {
          println!("<builtin>;");
        }
        TypeVariant::__LifetimeHolder(_) => {
          unreachable!()
        }
      }
    }
    for &(ref name, ref value) in &self.funcs {
      let (name, value) =
        (
          name.as_ref().expect("286 name"),
          value.as_ref().expect("286 val"),
        );
      print!("{} :: ", name);
      println!("() -> {} {{", value.ret_ty.0);

      println!("  locals: {{");
      for loc_ty in &value.locals {
        println!("    {},", loc_ty.0);
      }
      println!("  }}");

      println!("  bindings: {{");
      for (i, binding) in value.bindings.iter().enumerate() {
        if i == 0 {
          println!("    <return>: {}", (binding.1).0);
        } else {
          println!(
            "    {}_{}: {},",
            binding_name(&binding.0),
            i,
            (binding.1).0,
          );
        }
      }
      println!("  }}");

      for bb in &value.blks {
        println!("  bb{}: {{", bb.num.0);
        for stmt in &bb.stmts {
          let Statement { ref lhs, ref rhs } = *stmt;
          if lhs.0 == 0 {
            print!("    <return> = ");
          } else {
            print!("    ");
            let name = binding_name(
              &value.bindings[lhs.0 as usize].0,
            );
            print!("{}_{} = ", name, lhs.0);
          }
          match *rhs {
            Value::Literal(n) => {
              println!("literal {};", n);
            }
            Value::Reference(r) => {
              print_binding(&value.bindings, r);
              println!(";");
            }
            Value::Add(lhs, rhs) => {
              print_binding(&value.bindings, lhs);
              print!(" + ");
              print_binding(&value.bindings, rhs);
              println!(";");
            }
            Value::Call { callee } => {
              let name = match self.funcs[callee.0].0 {
                Some(ref name) => {
                  &**name
                }
                None => {
                  "<anonymous>"
                }
              };
              println!("{}();", name);
            }
          }
        }
        match bb.term {
          Terminator::Goto(blk) => {
            println!("    goto bb{};", blk.0);
          }
          Terminator::Return => {
            println!("    return;");
          }
        }
        println!("  }}");
      }
      println!("}}");
    }
  }
}
