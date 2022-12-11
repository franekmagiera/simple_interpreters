import simple_arithmetic.exceptions as exceptions
import simple_arithmetic.token as token
import typing


class Scanner:
    def scan(self, source: str) -> list[token.Token]:
        current = 0
        tokens = []

        while current < len(source):
            lexeme = source[current]
            match lexeme:
                case "+" | "-" | "*":
                    token = self.__scan_operator(lexeme)
                    tokens.append(token)
                    current += 1
                case string if string.isnumeric():
                    current, token = self.__scan_number(current, source)
                    tokens.append(token)
                case string if string.isspace():
                    current += 1
                case _:
                    raise exceptions.UnexpectedInputError(lexeme)

        return tokens

    def __scan_operator(
        self,
        lexeme: typing.Literal["+"]
        | typing.Literal["-"]
        | typing.Literal["*"],
    ) -> token.Token:
        match lexeme:
            case "+":
                type = token.TokenType.PLUS
            case "-":
                type = token.TokenType.MINUS
            case "*":
                type = token.TokenType.MUL
        return token.Token(type, lexeme)

    def __scan_number(
        self, start: int, source: str
    ) -> typing.Tuple[int, token.Token]:
        current = start
        while current < len(source) and source[current].isnumeric():
            current += 1
        num = source[start:current]
        return current, token.Token(token.TokenType.NUM, num)
