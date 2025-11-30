use crate::document::Span;

/// Тип директивы
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DirectiveType {
	Alias,
	Code,
	Dependencies,
}

/// Директива в документе
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Directive {
	/// Тип директивы
	pub kind: DirectiveType,
	/// Span всего объявления директивы
	pub span: Span,
	/// Span полезной нагрузки (содержимое директивы)
	pub payload_span: Span,
}

impl Directive {
	/// Создать директиву с указанным типом и spans
	pub fn new(kind: DirectiveType, span: Span, payload_span: Span) -> Self {
		Self {
			kind,
			span,
			payload_span,
		}
	}
}

