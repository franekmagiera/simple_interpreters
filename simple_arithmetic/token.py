import enum
import typing


class TokenType(enum.Enum):
    # Single character tokens.
    PLUS = "PLUS"
    MINUS = "MINUS"
    MUL = "MUL"
    # Literals.
    NUM = "NUMBER"


class Token(typing.NamedTuple):
    type: TokenType
    lexeme: str
