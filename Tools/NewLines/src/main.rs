//! Puts a blank line between neighbouring Rust statements, or imports, of different classes.

use std::{
	env, fs,
	io::{self, Write as _},
	process::ExitCode,
};

use ra_ap_syntax::{
	AstNode as _, Edition, SourceFile, SyntaxNode,
	ast::{BinaryOp, Expr, Item, NameRef, Stmt, StmtList, Use},
};

const PARSED: &str = "a file without syntax errors has this expression";

#[derive(Clone, Copy, PartialEq, Eq)]
enum Root {
	Core,
	Alloc,
	Std,
	Crate,
	Super,
	Other,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Class {
	Use(Root),
	Item,
	Declaration,
	Assignment,
	Call,
	ControlFlow,
	Terminator,
	Expression,
	Result,
}

fn root(import: &Use) -> Option<NameRef> {
	import.use_tree()?.path()?.first_segment()?.name_ref()
}

impl Root {
	fn from_use(import: &Use) -> Self {
		match root(import).as_ref().map(NameRef::text) {
			Some("core") => Self::Core,
			Some("alloc") => Self::Alloc,
			Some("std") => Self::Std,
			Some("crate") => Self::Crate,
			Some("super") => Self::Super,
			_ => Self::Other,
		}
	}
}

impl Class {
	fn from_expression(expression: Expr) -> Self {
		match expression {
			Expr::AwaitExpr(wrapper) => Self::from_expression(wrapper.expr().expect(PARSED)),
			Expr::ParenExpr(wrapper) => Self::from_expression(wrapper.expr().expect(PARSED)),
			Expr::TryExpr(wrapper) => Self::from_expression(wrapper.expr().expect(PARSED)),
			Expr::BinExpr(binary)
				if matches!(binary.op_kind(), Some(BinaryOp::Assignment { .. })) =>
			{
				Self::Assignment
			}
			Expr::CallExpr(_) | Expr::MacroExpr(_) | Expr::MethodCallExpr(_) => Self::Call,
			Expr::BlockExpr(_)
			| Expr::ForExpr(_)
			| Expr::IfExpr(_)
			| Expr::LoopExpr(_)
			| Expr::MatchExpr(_)
			| Expr::WhileExpr(_) => Self::ControlFlow,
			Expr::BecomeExpr(_)
			| Expr::BreakExpr(_)
			| Expr::ContinueExpr(_)
			| Expr::ReturnExpr(_)
			| Expr::YeetExpr(_)
			| Expr::YieldExpr(_) => Self::Terminator,
			Expr::ArrayExpr(_)
			| Expr::AsmExpr(_)
			| Expr::BinExpr(_)
			| Expr::CastExpr(_)
			| Expr::CfgPredExpr(_)
			| Expr::ClosureExpr(_)
			| Expr::FieldExpr(_)
			| Expr::FormatArgsExpr(_)
			| Expr::IncludeBytesExpr(_)
			| Expr::IndexExpr(_)
			| Expr::LetExpr(_)
			| Expr::Literal(_)
			| Expr::OffsetOfExpr(_)
			| Expr::PathExpr(_)
			| Expr::PrefixExpr(_)
			| Expr::RangeExpr(_)
			| Expr::RecordExpr(_)
			| Expr::RefExpr(_)
			| Expr::TupleExpr(_)
			| Expr::UnderscoreExpr(_) => Self::Expression,
		}
	}

	fn from_statement(statement: Stmt) -> Self {
		match statement {
			Stmt::ExprStmt(statement) => Self::from_expression(statement.expr().expect(PARSED)),
			Stmt::Item(Item::Use(import)) => Self::Use(Root::from_use(&import)),
			Stmt::Item(_) => Self::Item,
			Stmt::LetStmt(_) => Self::Declaration,
		}
	}

	/// A statement list holds statements, inner attributes and at most one expression, its result.
	fn from_block_child(node: &SyntaxNode) -> Option<Self> {
		match Stmt::cast(node.clone()) {
			Some(statement) => Some(Self::from_statement(statement)),
			None => Expr::can_cast(node.kind()).then_some(Self::Result),
		}
	}

	fn from_item(node: &SyntaxNode) -> Option<Self> {
		let import = Use::cast(node.clone())?;

		Some(Self::Use(Root::from_use(&import)))
	}
}

fn classes(previous: &SyntaxNode, node: &SyntaxNode) -> Option<(Class, Class)> {
	let in_block = node
		.parent()
		.is_some_and(|parent| StmtList::can_cast(parent.kind()));
	let class = if in_block {
		Class::from_block_child
	} else {
		Class::from_item
	};

	class(previous).zip(class(node))
}

/// The blank line goes after the line that ends the earlier statement, so it sits above comments
/// that lead into the later one.
fn blank_line_offset(gap: &str) -> Option<usize> {
	let line_end = gap.find('\n')?;
	let has_blank_line = gap
		.trim_end_matches([' ', '\t'])
		.lines()
		.skip(1)
		.map(str::trim)
		.any(str::is_empty);

	(!has_blank_line).then_some(line_end + 1)
}

fn blank_line_at(text: &str, node: &SyntaxNode) -> Option<usize> {
	let previous = node.prev_sibling()?;
	let (before, after) = classes(&previous, node)?;

	if before == after {
		return None;
	}

	let gap_start = usize::from(previous.text_range().end());
	let gap = &text[gap_start..usize::from(node.text_range().start())];

	Some(gap_start + blank_line_offset(gap)?)
}

/// Preorder visits statements in source order, so the insertions arrive in ascending order.
fn separate(text: &str, file: &SourceFile, output: &mut String) {
	let mut copied = 0;

	output.clear();

	for at in file
		.syntax()
		.descendants()
		.filter_map(|node| blank_line_at(text, &node))
	{
		output.push_str(&text[copied..at]);
		output.push('\n');

		copied = at;
	}

	output.push_str(&text[copied..]);
}

fn separate_file(path: &str, output: &mut String) -> Result<(), String> {
	let text = fs::read_to_string(path).map_err(|error| error.to_string())?;
	let parse = SourceFile::parse(&text, Edition::Edition2024);

	if let Some(error) = parse.errors().first() {
		return Err(error.to_string());
	}

	separate(&text, &parse.tree(), output);

	// Rewriting an unchanged file would change its modification time and make cargo rebuild.
	if output.len() != text.len() {
		fs::write(path, output.as_bytes()).expect("the file is writable");
	}

	Ok(())
}

fn main() -> ExitCode {
	let mut output = String::new();
	let mut failed = false;
	let mut stderr = io::stderr();

	for path in env::args().skip(1) {
		if let Err(error) = separate_file(&path, &mut output) {
			writeln!(stderr, "{path}: {error}").expect("stderr is writable");

			failed = true;
		}
	}

	ExitCode::from(u8::from(failed))
}
