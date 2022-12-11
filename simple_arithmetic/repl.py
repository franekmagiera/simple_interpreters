import simple_arithmetic.evaluator as evaluator
import simple_arithmetic.parser as parser
import simple_arithmetic.scanner as scanner
import signal
import sys
import types
import typing


def sig_int_handler(
    signal: int, stack_frame: typing.Optional[types.FrameType]
) -> None:
    print("\nGoodbye!")
    sys.exit()


signal.signal(signal.SIGINT, sig_int_handler)


class SimpleArithmetic:
    def __init__(
        self,
        scanner: scanner.Scanner,
        parser: parser.Parser,
        evaluator: evaluator.Evaluator,
    ) -> None:
        self.scanner = scanner
        self.parser = parser
        self.evaluator = evaluator

    def __read(self) -> str:
        return input(">>> ")

    def eval(self, source: str) -> int:
        tokens = self.scanner.scan(source)
        ast = self.parser.parse(tokens)
        result = self.evaluator.evaluate(ast)
        return result

    def repl(self) -> None:
        print("Simple arithmetic")
        while True:
            print(self.eval(self.__read()))
