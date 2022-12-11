from simple_arithmetic.evaluator import Evaluator
from simple_arithmetic.parser import Parser
from simple_arithmetic.repl import SimpleArithmetic
from simple_arithmetic.scanner import Scanner

SimpleArithmetic(Scanner(), Parser(), Evaluator()).repl()
