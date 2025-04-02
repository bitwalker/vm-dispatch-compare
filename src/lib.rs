use smallvec::SmallVec;

use std::ops::ControlFlow;

macro_rules! dispatch {
    ($context:ident) => {{
        let inst = unsafe { $context.code.get_unchecked($context.ip) };
        $context.ip += 1;
        (inst.execute)(&inst.opcode, $context)
    }}
}

pub trait Op {
    fn execute(&self, context: &mut VmContext) -> ControlFlow<Exit>;
    fn execute_nodispatch(&self, context: &mut VmContext) -> ControlFlow<Exit>;
}


#[derive(Copy, Clone)]
pub struct Begin;
impl Op for Begin {
    #[inline(always)]
    fn execute(&self, context: &mut VmContext) -> ControlFlow<Exit> {
        dispatch!(context)
    }
    #[inline(always)]
    fn execute_nodispatch(&self, _context: &mut VmContext) -> ControlFlow<Exit> {
        ControlFlow::Continue(())
    }
}

#[derive(Copy, Clone)]
pub struct Nop;
impl Op for Nop {
    #[inline(always)]
    fn execute(&self, context: &mut VmContext) -> ControlFlow<Exit> {
        dispatch!(context)
    }
    #[inline(always)]
    fn execute_nodispatch(&self, _context: &mut VmContext) -> ControlFlow<Exit> {
        ControlFlow::Continue(())
    }
}

#[derive(Copy, Clone)]
pub struct Stop;
impl Op for Stop {
    #[inline(always)]
    fn execute(&self, _context: &mut VmContext) -> ControlFlow<Exit> {
        ControlFlow::Break(Exit::Stop)
    }
    #[inline(always)]
    fn execute_nodispatch(&self, _context: &mut VmContext) -> ControlFlow<Exit> {
        ControlFlow::Break(Exit::Stop)
    }
}

#[derive(Copy, Clone)]
pub struct Push(u32);
impl Op for Push {
    #[inline(always)]
    fn execute(&self, context: &mut VmContext) -> ControlFlow<Exit> {
        context.stack.push(self.0);
        dispatch!(context)
    }
    #[inline(always)]
    fn execute_nodispatch(&self, context: &mut VmContext) -> ControlFlow<Exit> {
        context.stack.push(self.0);
        ControlFlow::Continue(())
    }
}

#[derive(Copy, Clone)]
pub struct Add;
impl Op for Add {
    #[inline(always)]
    fn execute(&self, context: &mut VmContext) -> ControlFlow<Exit> {
        assert!(context.stack.len() > 1, "expected operand");
        let b = unsafe { context.stack.pop().unwrap_unchecked() };
        let a = unsafe { context.stack.pop().unwrap_unchecked() };
        context.stack.push(a.wrapping_add(b));
        dispatch!(context)
    }
    #[inline(always)]
    fn execute_nodispatch(&self, context: &mut VmContext) -> ControlFlow<Exit> {
        assert!(context.stack.len() > 1, "expected operand");
        let b = unsafe { context.stack.pop().unwrap_unchecked() };
        let a = unsafe { context.stack.pop().unwrap_unchecked() };
        context.stack.push(a.wrapping_add(b));
        ControlFlow::Continue(())
    }
}

#[derive(Copy, Clone)]
pub struct Mul;
impl Op for Mul {
    #[inline(always)]
    fn execute(&self, context: &mut VmContext) -> ControlFlow<Exit> {
        assert!(context.stack.len() > 1, "expected operand");
        let b = unsafe { context.stack.pop().unwrap_unchecked() };
        let a = unsafe { context.stack.pop().unwrap_unchecked() };
        context.stack.push(a.wrapping_mul(b));
        dispatch!(context)
    }
    #[inline(always)]
    fn execute_nodispatch(&self, context: &mut VmContext) -> ControlFlow<Exit> {
        assert!(context.stack.len() > 1, "expected operand");
        let b = unsafe { context.stack.pop().unwrap_unchecked() };
        let a = unsafe { context.stack.pop().unwrap_unchecked() };
        context.stack.push(a.wrapping_mul(b));
        ControlFlow::Continue(())
    }
}

#[derive(Copy, Clone)]
pub struct If {
    /// The offset of the instruction to jump to if the condition is true
    pub if_true: u32,
    /// The offset of the instruction to jump to if the condition is false
    pub if_false: u32,
}
impl Op for If {
    #[inline(always)]
    fn execute(&self, context: &mut VmContext) -> ControlFlow<Exit> {
        assert!(!context.stack.is_empty(), "expected operand");
        let cond = unsafe { context.stack.pop().unwrap_unchecked() };
        if cond == 0 {
            context.ip = self.if_false as usize;
        } else {
            context.ip = self.if_true as usize;
        }
        dispatch!(context)
    }
    #[inline(always)]
    fn execute_nodispatch(&self, context: &mut VmContext) -> ControlFlow<Exit> {
        assert!(!context.stack.is_empty(), "expected operand");
        let cond = unsafe { context.stack.pop().unwrap_unchecked() };
        if cond == 0 {
            context.ip = self.if_false as usize;
        } else {
            context.ip = self.if_true as usize;
        }
        ControlFlow::Continue(())
    }
}

#[derive(Clone)]
pub enum Inst {
    Begin,
    Nop,
    Stop,
    Push(u32),
    If(If),
    Add,
    Mul,
}

pub struct CompiledInst {
    opcode: Opcode,
    execute: fn(&Opcode, &mut VmContext) -> ControlFlow<Exit>,
}

pub union Opcode {
    begin: Begin,
    nop: Nop,
    stop: Stop,
    push: Push,
    r#if: If,
    add: Add,
    mul: Mul,
}

macro_rules! opcode_impl {
    ($($opcode:ident),*) => {
        $(
            #[inline(never)]
            pub fn $opcode(&self, context: &mut VmContext) -> ControlFlow<Exit> {
                unsafe { self.$opcode }.execute(context)
            }
        )*
    }
}

impl Opcode {
    pub fn compile(inst: Inst) -> CompiledInst {
        match inst {
            Inst::Begin => CompiledInst {
                opcode: Self { begin: Begin },
                execute: Self::begin,
            },
            Inst::Nop => CompiledInst {
                opcode: Self { nop: Nop },
                execute: Self::nop,
            },
            Inst::Stop => CompiledInst {
                opcode: Self { stop: Stop },
                execute: Self::stop,
            },
            Inst::Push(value) => CompiledInst {
                opcode: Self { push: Push(value) },
                execute: Self::push,
            },
            Inst::If(r#if) => CompiledInst {
                opcode: Self { r#if },
                execute: Self::r#if,
            },
            Inst::Add => CompiledInst {
                opcode: Self { add: Add },
                execute: Self::add,
            },
            Inst::Mul => CompiledInst {
                opcode: Self { mul: Mul },
                execute: Self::mul,
            },
        }
    }
    
    opcode_impl!(begin, nop, stop, push, r#if, add, mul);
}


pub struct SystemError;

pub enum Exit {
    Stop,
    Error(SystemError),
}

#[derive(Default)]
pub struct VmContext<'a> {
    ip: usize,
    code: &'a [CompiledInst],
    stack: SmallVec<[u32; 16]>,
}

#[inline(never)]
pub fn computed_goto(code: &[CompiledInst]) -> ControlFlow<Exit> {
    // The VmContext represents whatever state is needed by opcode implementations
    let mut context = VmContext {
        ip: 0,
        code,
        stack: Default::default(),
    };
    
    let ctx = &mut context;
    
    dispatch!(ctx)
}

#[inline(never)]
pub fn switch_based(code: &[Inst]) -> ControlFlow<Exit> {
    // The VmContext represents whatever state is needed by opcode implementations
    let mut context = VmContext {
        ip: 0,
        code: Default::default(),
        stack: Default::default(),
    };

    let ctx = &mut context;
    
    loop {
        let opcode = unsafe { code.get_unchecked(ctx.ip) };
        ctx.ip += 1;
        match opcode {
            Inst::Begin => Begin.execute_nodispatch(ctx),
            Inst::Nop => Nop.execute_nodispatch(ctx),
            Inst::Stop => Stop.execute_nodispatch(ctx),
            Inst::Push(val) => Push(*val).execute_nodispatch(ctx),
            Inst::Add => Add.execute_nodispatch(ctx),
            Inst::Mul => Mul.execute_nodispatch(ctx),
            Inst::If(r#if) => r#if.execute_nodispatch(ctx),
        }?;
    }
}
