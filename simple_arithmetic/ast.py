from simple_arithmetic.token import Token, TokenType
import typing

RT = typing.TypeVar("RT")


class Node(typing.Protocol[RT]):
    def accept(self, visitor: "NodeVisitor[RT]") -> RT:
        ...


class BinOp(Node[RT]):
    """Binary operation"""

    def __init__(
        self,
        left: Node,
        operator: TokenType,
        right: Node,
    ):
        self.left = left
        self.operator = operator
        self.right = right

    def accept(self, visitor: "NodeVisitor[RT]") -> RT:
        return visitor.visit_BinOp(self)


class Literal(Node[RT]):
    def __init__(self, value: Token):
        self.value = value

    def accept(self, visitor: "NodeVisitor[RT]") -> RT:
        return visitor.visit_Literal(self)


RT_co = typing.TypeVar("RT_co", covariant=True)


class NodeVisitor(typing.Protocol[RT_co]):
    def visit_BinOp(self, bin_op: BinOp) -> RT_co:
        ...

    def visit_Literal(self, literal: Literal) -> RT_co:
        ...


class Formatter(NodeVisitor[str]):
    """Formats the AST as a LISP-style tree list."""

    def visit_BinOp(self, bin_op: BinOp) -> str:
        return (
            f"({bin_op.operator.value} "
            f"{bin_op.left.accept(self)} "
            f"{bin_op.right.accept(self)})"
        )

    def visit_Literal(self, literal: Literal) -> str:
        return literal.value.lexeme
