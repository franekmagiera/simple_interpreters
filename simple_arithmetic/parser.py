import simple_arithmetic.ast as ast
import simple_arithmetic.exceptions as exceptions
import simple_arithmetic.token as token
import collections


class Parser:
    """Recursive descent parser."""

    def parse(self, tokens: collections.abc.Sequence[token.Token]) -> ast.Node:
        self.__tokens = tokens
        self.__current = 0
        return self.__term()

    def __term(self) -> ast.Node:
        term = self.__factor()

        while self.__match(token.TokenType.PLUS, token.TokenType.MINUS):
            operator = self.__previous().type
            right = self.__factor()
            term = ast.BinOp(term, operator, right)

        return term

    def __factor(self) -> ast.Node:
        factor: ast.BinOp | ast.Literal = self.__literal()

        while self.__match(token.TokenType.MUL):
            operator = self.__previous().type
            right = self.__literal()
            factor = ast.BinOp(factor, operator, right)

        return factor

    def __literal(self) -> ast.Literal:
        if token.TokenType.NUM is self.__tokens[self.__current].type:
            current_token = self.__peek()
            self.__advance()
            return ast.Literal(current_token)
        raise exceptions.ParseError("Wrong syntax")

    def __match(self, *token_types: token.TokenType) -> bool:
        for token_type in token_types:
            if self.__expected(token_type):
                self.__advance()
                return True
        return False

    def __expected(self, token_type: token.TokenType) -> bool:
        if self.__is_last():
            return False
        return self.__peek().type is token_type

    def __advance(self) -> None:
        if not self.__is_last():
            self.__current += 1

    def __is_last(self) -> bool:
        return self.__current == len(self.__tokens) - 1

    def __peek(self) -> token.Token:
        return self.__tokens[self.__current]

    def __previous(self) -> token.Token:
        return self.__tokens[self.__current - 1]
