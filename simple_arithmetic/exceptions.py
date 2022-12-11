class UnexpectedInputError(ValueError):
    def __init__(self, message: str) -> None:
        super().__init__(f"Unexpected input: {message}")


class ParseError(Exception):
    def __init__(self, message: str) -> None:
        super().__init__(message)


class EvaluationError(Exception):
    def __init__(self, message: str) -> None:
        super().__init__(message)
