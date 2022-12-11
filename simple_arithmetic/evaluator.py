from simple_arithmetic.ast import BinOp, Literal, Node, NodeVisitor
from simple_arithmetic.exceptions import EvaluationError
from simple_arithmetic.token import TokenType


class Evaluator(NodeVisitor[int]):
    def evaluate(self, ast: Node) -> int:
        return ast.accept(self)

    def visit_BinOp(self, bin_op: BinOp) -> int:
        left = bin_op.left.accept(self)
        right = bin_op.right.accept(self)
        match bin_op.operator:
            case TokenType.PLUS:
                return left + right
            case TokenType.MINUS:
                return left - right
            case TokenType.MUL:
                return left * right
            case _:
                raise EvaluationError(f"Unknown operator: {bin_op.operator}")

    def visit_Literal(self, literal: Literal) -> int:
        return int(literal.value.lexeme)
