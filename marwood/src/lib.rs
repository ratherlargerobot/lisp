pub mod cell;
pub mod lex;
pub mod number;
pub mod parse;
pub mod vm;

#[cfg(test)]
mod integration_test {
    use crate::cell::Cell;
    use crate::lex;
    use crate::parse;
    use crate::vm::Error::{
        ExpectedPairButFound, InvalidArgs, InvalidDefineSyntax, InvalidNumArgs, InvalidProcedure,
        InvalidStringIndex, InvalidSyntax, InvalidUsePrimitive, InvalidVectorIndex, UnquotedNil,
        VariableNotBound,
    };
    use crate::vm::Vm;

    macro_rules! evals {
        ($($lhs:expr => $rhs:expr),+) => {{
            let mut vm = Vm::new();
             $(
                assert_eq!(vm.eval(&parse!($lhs)), Ok(match $rhs {
                    "#<void>" => Cell::Void,
                    _ => parse!($rhs)
                }));
             )+
        }};
    }

    macro_rules! fails {
        ($($lhs:expr => $rhs:expr),+) => {{
            let mut vm = Vm::new();
             $(
                assert_eq!(vm.eval(
                    &parse!($lhs)
                ), Err($rhs));
             )+
        }};
    }

    #[test]
    fn eval_literal() {
        evals![
           "1" => "1",
           "-10" => "-10",
           "#t" => "#t",
           "#f" => "#f",
           "'()" => "()"
        ];

        fails!["foo" => VariableNotBound("foo".into()),
               "()" => UnquotedNil];
    }

    #[test]
    fn eval_string_char_literals() {
        evals![
           "#\\a" => "#\\a",
           "#\\A" => "#\\A",
           "#\\)" => "#\\)",
           "#\\(" => "#\\(",
           "#\\newline" => "#\\newline",
           "#\\space" => "#\\space",
           "#\\ " => "#\\space"
        ];

        evals![
            r#""foo \"bar\" baz""# => r#""foo \"bar\" baz""#
        ]
    }

    #[test]
    fn char_procedures() {
        evals![
            "(integer->char (char->integer #\\a))" => "#\\a",
            "(integer->char (char->integer #\\𒀀))" => "#\\𒀀",
            "(integer->char (char->integer #\\alarm))" => "#\\alarm",
            "(integer->char (char->integer #\\backspace))" => "#\\backspace",
            "(integer->char (char->integer #\\delete))" => "#\\delete",
            "(integer->char (char->integer #\\escape))" => "#\\escape",
            "(integer->char (char->integer #\\newline))" => "#\\newline",
            "(integer->char (char->integer #\\null))" => "#\\null",
            "(integer->char (char->integer #\\return))" => "#\\return",
            "(integer->char (char->integer #\\space))" => "#\\space",
            "(integer->char (char->integer #\\tab))" => "#\\tab"
        ];
        fails![
            "(integer->char #xffffffffff)" => InvalidSyntax("4294967295 is not valid unicode".into())
        ];
        evals![
            "(char-alphabetic? #\\a)" => "#t",
            "(char-alphabetic? #\\A)" => "#t",
            "(char-alphabetic? #\\1)" => "#f",
            "(char-numeric? #\\1)" => "#t",
            "(char-numeric? #\\a)" => "#f",
            "(char-whitespace? #\\space)" => "#t",
            "(char-whitespace? #\\tab)" => "#t",
            "(char-upper-case? #\\a)" => "#f",
            "(char-upper-case? #\\A)" => "#t",
            "(char-lower-case? #\\a)" => "#t",
            "(char-lower-case? #\\A)" => "#f"
        ];

        evals![
            "(char=? #\\a #\\a)" => "#t",
            "(char=? #\\a #\\b)" => "#f",
            "(char=? #\\A #\\a)" => "#f",
            "(char-ci=? #\\A #\\a)" => "#t",

            "(char<? #\\a #\\b)" => "#t",
            "(char<? #\\a #\\B)" => "#f",
            "(char-ci<? #\\a #\\B)" => "#t",
            "(char<? #\\a #\\a)" => "#f",
            "(char<=? #\\a #\\a)" => "#t",

            "(char>? #\\b #\\a)" => "#t",
            "(char-ci>? #\\b #\\A)" => "#t",
            "(char>? #\\a #\\a)" => "#f",
            "(char>=? #\\a #\\a)" => "#t"
        ];

        evals![
            "(char-upcase #\\a)" => "#\\A",
            "(char-upcase #\\λ)" => "#\\Λ",

            "(char-downcase #\\A)" => "#\\a",
            "(char-downcase #\\Λ)" => "#\\λ",

            // ß -> SS
            "(char-downcase #\\ß)" => "#\\ß",
            "(char-foldcase #\\Σ)" => "#\\σ"
        ];

        evals![
            "(digit-value #\\9)" => "9",
            "(digit-value #\\a)" => "#f"
        ];
    }

    #[test]
    fn string_procedures() {
        evals![
            "(make-string 5)" => "\"\0\0\0\0\0\"",
            "(make-string 5 #\\a)" => "\"aaaaa\""
        ];
        evals![
            "(string #\\a)" => "\"a\"",
            "(string #\\a #\\b)" => "\"ab\""
        ];
        evals![
            "(string-length \"foo\")" => "3",
            "(string-length \"🐶\")" => "1"
        ];
        evals![
            "(string-ref \"o🐶o\" 0)" => "#\\o",
            "(string-ref \"o🐶o\" 1)" => "#\\🐶",
            "(string-ref \"o🐶o\" 2)" => "#\\o"
        ];
        fails!["(string-ref \"o🐶o\" 3)" => InvalidStringIndex(3, 2)];
        evals![
            "(define owo \"o🐶o\")" => "#<void>",
            "(string-set! owo 0 #\\f)" => "#<void>",
            "owo" => "\"f🐶o\"",
            "(define owo \"o🐶o\")" => "#<void>",
            "(string-set! owo 1 #\\w)" => "#<void>",
            "owo" => "\"owo\"",
            "(define owo \"o🐶o\")" => "#<void>",
            "(string-set! owo 2 #\\f)" => "#<void>",
            "owo" => "\"o🐶f\""
        ];
        fails!["(string-set! \"o🐶o\" 3 #\\f)" => InvalidStringIndex(3, 2)];
        evals![
            "(string-append \"foo\")" => "\"foo\"",
            "(string-append \"foo \" \"bar\")" => "\"foo bar\""
        ];

        evals![
            "(string=? \"foo\" \"foo\")" => "#t",
            "(string<? \"boo\" \"foo\")" => "#t",
            "(string>? \"foo\" \"boo\")" => "#t",
            "(string<=? \"foo\" \"foo\")" => "#t",
            "(string<=? \"boo\" \"foo\")" => "#t",
            "(string>=? \"foo\" \"foo\")" => "#t",
            "(string>=? \"foo\" \"boo\")" => "#t",
            "(string-ci=? \"Foo\" \"foo\")" => "#t",
            "(string-ci<? \"Boo\" \"foo\")" => "#t",
            "(string-ci>? \"Foo\" \"boo\")" => "#t",
            "(string-ci<=? \"Foo\" \"foo\")" => "#t",
            "(string-ci<=? \"Boo\" \"foo\")" => "#t",
            "(string-ci>=? \"Foo\" \"foo\")" => "#t",
            "(string-ci>=? \"Foo\" \"boo\")" => "#t"
        ];
        evals![
            "(string-downcase \"FOO\")" => "\"foo\"",
            "(string-upcase \"foo\")" => "\"FOO\"",
            "(string-foldcase \"FOO\")" => "\"foo\""
        ];

        evals![
            "(substring \"o🐶o\" 0 0)" => "\"\"",
            "(substring \"o🐶o\" 0 1)" => "\"o\"",
            "(substring \"o🐶o\" 0 2)" => "\"o🐶\"",
            "(substring \"o🐶o\" 0 3)" => "\"o🐶o\"",
            "(substring \"o🐶o\" 1 2)" => "\"🐶\"",
            "(substring \"o🐶o\" 2 3)" => "\"o\"",
            "(substring \"o🐶o\" 3 3)" => "\"\""
        ];
        fails![
            "(substring \"o🐶o\" 2 1)" => InvalidSyntax("invalid substring indices: end < start".into())
        ];

        evals![
            "(string-copy \"o🐶o\")" => "\"o🐶o\"",
            "(string-copy \"o🐶o\" 0)" => "\"o🐶o\"",
            "(string-copy \"o🐶o\" 1)" => "\"🐶o\"",
            "(string-copy \"o🐶o\" 2)" => "\"o\"",
            "(string-copy \"o🐶o\" 3)" => "\"\"",
            "(string-copy \"o🐶o\" 0 0)" => "\"\"",
            "(string-copy \"o🐶o\" 0 1)" => "\"o\"",
            "(string-copy \"o🐶o\" 0 2)" => "\"o🐶\"",
            "(string-copy \"o🐶o\" 0 3)" => "\"o🐶o\"",
            "(string-copy \"o🐶o\" 1 2)" => "\"🐶\"",
            "(string-copy \"o🐶o\" 2 3)" => "\"o\"",
            "(string-copy \"o🐶o\" 3 3)" => "\"\""
        ];

        evals![
            "(string->list \"o🐶o\")" => "(#\\o #\\🐶 #\\o)",
            "(string->list \"o🐶o\" 0)" => "(#\\o #\\🐶 #\\o)",
            "(string->list \"o🐶o\" 1)" => "(#\\🐶 #\\o)",
            "(string->list \"o🐶o\" 2)" => "(#\\o)",
            "(string->list \"o🐶o\" 3)" => "()",
            "(string->list \"o🐶o\" 0 1)" => "(#\\o)",
            "(string->list \"o🐶o\" 0 2)" => "(#\\o #\\🐶)",
            "(string->list \"o🐶o\" 0 3)" => "(#\\o #\\🐶 #\\o)",
            "(string->list \"o🐶o\" 1 2)" => "(#\\🐶)",
            "(string->list \"o🐶o\" 2 3)" => "(#\\o)",
            "(string->list \"o🐶o\" 3 3)" => "()"
        ];

        evals![
            "(list->string '(#\\o #\\🐶 #\\o))" => "\"o🐶o\""
        ];
        fails![
            "(list->string '(10))" => InvalidSyntax("list->string expected char but found 10".into())
        ];

        evals![
            "(list->string (string->list \"o🐶o\"))" => "\"o🐶o\""
        ];

        evals![
            "(define owo \"o🐶o\")" => "#<void>",
            "(string-fill! owo #\\z)" => "#<void>",
            "owo" => "\"zzz\"",
            "(define owo \"o🐶o\")" => "#<void>",
            "(string-fill! owo #\\z 1)" => "#<void>",
            "owo" => "\"ozz\"",
            "(define owo \"o🐶o\")" => "#<void>",
            "(string-fill! owo #\\z 1 2)" => "#<void>",
            "owo" => "\"ozo\""
        ];
    }

    #[test]
    fn symbol_procedures() {
        evals!["(string->symbol \"12foo\")" => "\\x31;2foo",
               "(string->symbol \" foo\")" => "\\x20;foo",
               "(symbol->string (string->symbol \"12foo\"))" =>  "\"12foo\"",
               "(symbol->string (string->symbol \" foo\"))" =>  "\" foo\""
        ];
        evals!["(symbol=? 'foo 'foo 'foo)" => "#t",
               "(symbol=? 'foo 'foo 'bar 'foo)" => "#f"
        ];
    }

    #[test]
    fn eval_quote() {
        evals![
            "'1" => "1",
            "'#t" => "#t",
            "'#f" => "#f",
            "'()" => "()",
            "'(1 2 3)" => "(1 2 3)"
        ];
    }

    #[test]
    fn car_and_cdr() {
        evals![
            "(car '(1 2 3))" => "1",
            "(cdr '(1 2 3))" => "(2 3)"
        ];
        fails!["(car 1)" => ExpectedPairButFound("1".into()),
               "(cdr 1)" => ExpectedPairButFound("1".into())
        ];
    }

    #[test]
    fn cons() {
        evals![
            "(cons 1 2)" => "(1 . 2)",
            "(cons '(1 2) '(3 4))" => "((1 2) . (3 4))",
            "(cons 1 (cons 2 (cons 3 '())))" => "(1 2 3)"
        ];
        fails!["(cons 1)" => InvalidNumArgs("cons".into()),
               "(cons 1 2 3)" => InvalidNumArgs("cons".into())
        ];
    }

    #[test]
    fn numerical_prefixes() {
        evals![
            "#d255" => "255",
            "#xFF" => "255",
            "#xff" => "255",
            "#e#xff" => "255",
            "#i#xff" => "255.0",
            "#b11111111" => "255",
            "#b11.11" => "3.75",
            "#i#xff" => "255.0",
            "#o10" => "8",
            "#e#o10" => "8",
            "#i#o10" => "8.0",
            "1e21" => "1e21",
            "1E21" => "1e21"
        ]
    }

    #[test]
    fn exact_vs_inexact() {
        evals![
            "#i1/2" => "0.5",
            "#e0.5" => "1/2"
        ];
        evals![
            "(exact->inexact 1/2)" => "0.5",
            "(inexact->exact 0.5)" => "1/2"
        ];
    }

    #[test]
    fn arithmetic() {
        evals![
            "(+)" => "0",
            "(+ 10)" => "10",
            "(+ 10 20)" => "30",
            "(+ 10 20 30)" => "60",
            "(+ (+ 100 100) (+ 25 25) 50)" => "300",
            "(- 10)" => "-10",
            "(- 100 10)" => "90",
            "(- 1000 100 10)" => "890",
            "(*)" => "1",
            "(* 10)" => "10",
            "(* 10 10)" => "100",
            "(* 10 10 10)" => "1000"
        ];
        fails!["(+ '(10))" => InvalidArgs("+".into(), "number".into(), "(10)".into()),
               "(+ 10 '(10))" => InvalidArgs("+".into(), "number".into(), "(10)".into()),
               "(-)" => InvalidNumArgs("-".into())
        ];

        evals![
            "(/ 10 5)" => "2",
            "(/ 10 3)" => "10/3"
        ];
        fails![
            "(/ 10 0)" => InvalidSyntax("/ is undefined for 0".into())
        ];

        evals![
            "(quotient 10 3)" => "3",
            "(remainder 10 3)" => "1",
            "(quotient 10/1 3/1)" => "3",
            "(remainder 10/1 3/1)" => "1"
        ];

        fails![
            "(remainder 10 0)" => InvalidSyntax("remainder is undefined for 0".into()),
            "(quotient 10 0)" => InvalidSyntax("quotient is undefined for 0".into())
        ];

        evals![
            "(modulo -21 4)" => "3"
        ];
    }

    #[test]
    fn abs() {
        evals![
            "(abs -10)" => "10",
            "(abs  10)" => "10",
            "(abs -10.0)" => "10",
            "(abs  10.0)" => "10",
            "(abs -1/2)" => "1/2",
            "(abs  1/2)" => "1/2"
        ];
    }

    #[test]
    fn eqv() {
        evals![
            "(define foo '(1 2 3))" => "#<void>",
            "(define bar '(1 2 3))" => "#<void>",
            "(define baz foo)" => "#<void>",
            "(eq? foo bar)" => "#f",
            "(eq? bar baz)" => "#f",
            "(eq? foo baz)" => "#t",
            "(eq? (cdr foo) (cdr baz))" => "#t",
            "(eq? (cons foo bar) (cons foo bar))" => "#t"
        ];

        evals![
            "(eq? 0 0)" => "#t",
            "(eq? #\\a #\\a)" => "#t",
            "(eq? '() '())" => "#t",
            "(eq? #f #f)" => "#t",
            "(eq? #t #t)" => "#t",
            "(eq? 'foo 'foo)" => "#t",
            "(eq? '(1 2 3) '(1 2 3))" => "#f",
            "(eq? #(1 2 3) #(1 2 3))" => "#f"
        ];
    }

    #[test]
    fn equal() {
        evals![
            "(define foo '(1 2 3))" => "#<void>",
            "(define bar '(1 2 3))" => "#<void>",
            "(define baz foo)" => "#<void>",
            "(equal? foo bar)" => "#t",
            "(equal? bar baz)" => "#t",
            "(equal? foo baz)" => "#t",
            "(equal? (cdr foo) (cdr baz))" => "#t",
            "(equal? (cons foo bar) (cons foo bar))" => "#t",
            "(equal? 0 0)" => "#t",
            "(equal? '() '())" => "#t",
            "(equal? #f #f)" => "#t",
            "(equal? #t #t)" => "#t",
            "(equal? 'foo 'foo)" => "#t",
            "(equal? #(1 2 3) #(1 2 3))" => "#t",
            "(equal? \"foo\" \"foo\")" => "#t",
            "(equal? #\\a #\\a)" => "#t"
        ];
    }

    #[test]
    fn not() {
        evals![
          "(not #t)" => "#f",
          "(not #f)" => "#t",
          "(not 'apples)" => "#f"
        ];
    }

    #[test]
    fn unary_predicate() {
        evals![
            "(number? 10)" => "#t",
            "(number? '10)" => "#t",
            "(number? 'apples)" => "#f"
        ];

        evals![
            "(boolean? #t)" => "#t",
            "(boolean? #f)" => "#t",
            "(boolean? '#t)" => "#t",
            "(boolean? '#f)" => "#t",
            "(boolean? 10)" => "#f"
        ];

        evals![
            "(symbol? 'apples)" => "#t",
            "(symbol? 10)" => "#f"
        ];

        evals![
            "(string? \"foo\")" => "#t",
            "(symbol? 10)" => "#f"
        ];

        evals![
            "(char? #\\a)" => "#t",
            "(char? 10)" => "#f"
        ];

        evals![
            "(symbol? 'apples)" => "#t",
            "(symbol? 10)" => "#f"
        ];
        evals![
            "(null? '())" => "#t",
            "(null? (cdr '(apples)))" => "#t",
            "(null? #f)" => "#f"
        ];

        evals![
            "(procedure? (lambda (x) x))" => "#t",
            "(define identity (lambda (x) x))" => "#<void>",
            "(procedure? identity)" => "#t"
        ];

        evals![
            "(pair? '(1 2 3))" => "#t",
            "(pair? '(1. 2))" => "#t",
            "(pair? (cons 1 2))" => "#t",
            "(pair? 'apples)" => "#f"
        ];

        evals![
            "(list? '())" => "#t",
            "(list? '(1 2))" => "#t",
            "(list? '(1 3 4))" => "#t",
            "(list? '(1 2 . 3))" => "#f"
        ];

        evals![
            "(vector? #(1 2 3))" => "#t"
        ];
    }

    #[test]
    fn if_expressions() {
        evals![
            "(if (list? '(1 2 3)) 'yep 'nope)" => "yep",
            "(if (list? #t) 'yep 'nope)" => "nope",
            "(if (list? '(1 2 3)) 'yep)" => "yep",
            "(if (list? #t) 'yep)" => "#<void>"
        ];
    }

    #[test]
    fn gc_cleans_intern_map() {
        evals![
            "'foo" => "foo",
            "'foo" => "foo"
        ]
    }

    #[test]
    fn invalid_procedure_calls() {
        fails![
            "(1 2 3)" => InvalidProcedure("1".to_string()),
            "((+ 1 1) 2 3)" => InvalidProcedure("2".to_string())
        ];
    }

    #[test]
    fn lambdas() {
        evals![
            "((lambda () 10))" => "10",
            "(+ ((lambda () 50)) ((lambda () 100)))" => "150"
        ];
    }

    #[test]
    fn tge_capturing_lambda() {
        evals![
            "(define x 100)" => "#<void>",
            "(define y 50)" => "#<void>",
            "((lambda () x))" => "100",
            "(+ ((lambda () x)) ((lambda () y)))" => "150",
            "(define x 1000)" => "#<void>",
            "(define y 500)" => "#<void>",
            "(+ ((lambda () x)) ((lambda () y)))" => "1500"
        ];
    }

    #[test]
    fn lambda_with_args() {
        // Identity
        evals![
            "(define identity (lambda (x) x))" => "#<void>",
            "(identity '(1 2 3))" => "(1 2 3)"
        ];

        // Two arg
        evals![
            "(define make-pair (lambda (x y) (cons x y)))" => "#<void>",
            "(make-pair 'apples 'bananas)" => "(apples . bananas)"
        ];

        // Mixed arg and global
        evals![
            "(define x 100)" => "#<void>",
            "(define add-to-x (lambda (y) (+ x y)))" => "#<void>",
            "(add-to-x 50)" => "150"
        ];
    }

    #[test]
    fn lambda_operator_is_expression() {
        evals![
            "(define proc (lambda () add))" => "#<void>",
            "(define add (lambda (x y) (+ x y)))" => "#<void>",
            "((proc) 1 2)" => "3"
        ];
    }

    #[test]
    fn iof_argument_capture() {
        evals![
            "(define make-adder (lambda (x) (lambda (y) (+ x y))))" => "#<void>",
            "(define add-10 (make-adder 10))" => "#<void>",
            "(add-10 20)" => "30"
        ];
    }

    #[test]
    fn iof_environment_capture() {
        evals![
            "(define make-make-adder (lambda (x) (lambda (y) (lambda (z) (+ x y z)))))" => "#<void>",
            "(define make-adder (make-make-adder 1000))" => "#<void>",
            "(define add-1000-100 (make-adder 100))" => "#<void>",
            "(add-1000-100 10)" => "1110"
        ];

        evals![
            "(define (make-make-adder x) (lambda (y) (lambda (z) (+ x y z))))" => "#<void>",
            "(define make-adder (make-make-adder 1000))" => "#<void>",
            "(define add-1000-100 (make-adder 100))" => "#<void>",
            "(add-1000-100 10)" => "1110"
        ];
    }

    #[test]
    fn iof_environment_capture_with_first_class_procedure() {
        evals![
            "(define make-adder-adder (lambda (adder num) (lambda (n) (+ ((adder num) n)))))" => "#<void>",
            "((make-adder-adder (lambda (i) (lambda (j) (+ i j))) 100) 1000)" => "1100"
        ];
    }

    #[test]
    fn define_special_forms() {
        evals![
            "(define (make-adder x) (lambda (y) (+ x y)))" => "#<void>",
            "(define add-10 (make-adder 10))" => "#<void>",
            "(add-10 20)" => "30"
        ];
    }

    #[test]
    fn vararg() {
        evals![
            "((lambda l l))" => "()",
            "((lambda l l) 10)" => "(10)",
            "((lambda l l) 10 20)" => "(10 20)"
        ];

        evals![
            "(define (list . a) a)" => "#<void>",
            "(list)" => "()",
            "(list 10)" => "(10)",
            "(list 10 20)" => "(10 20)"
        ];

        evals![
            "((lambda (x . y) (cons x y)) 10)" => "(10 . ())",
            "((lambda (x . y) (cons x y)) 10 20)" => "(10 . (20))",
            "((lambda (x y . z) (cons (+ x y) z)) 10 20)" => "(30 . ())",
            "((lambda (x y . z) (cons (+ x y) z)) 10 20 30)" => "(30 . (30))",
            "((lambda (x y . z) (cons (+ x y) z)) 10 20 30 40)" => "(30 . (30 40))"
        ];
    }

    #[test]
    fn tail_recursive() {
        evals![r#"(define (nth l n)
                    (if (null? l) '()
                        (if (eq? n 0) (car l)
                            (nth (cdr l) (- n 1)))))"# => "#<void>",
               "(nth '(1 2 3 4 5) 4)" => "5",
               "(nth '(1 2 3 4 5) 5)" => "()"
        ];
    }

    #[test]
    fn disallow_aliasing_syntactic_symbol() {
        fails!["(define if 42)" => InvalidUsePrimitive("if".into())];
        fails!["(define my-if if)" => InvalidUsePrimitive("if".into())];
        fails!["(lambda (if) 42)" => InvalidUsePrimitive("if".into())];
    }

    #[test]
    fn or() {
        evals!["(or 5)" => "5"];
        evals!["(or #f 5)" => "5"];
        evals!["(or (eq? 1 2) 'apples)" => "apples"];
    }

    #[test]
    fn and() {
        evals!["(and 5)" => "5"];
        evals!["(and #t 5)" => "5"];
        evals!["(and #f 5)" => "#f"];
        evals!["(and #t #t 5)" => "5"];
    }

    #[test]
    fn begin() {
        evals!["(begin (+ 10 10) (+ 20 20) (+ 5 5))" => "10"];
    }

    #[test]
    fn unless() {
        evals!["(unless #t 10)" => "#<void>"];
        evals!["(unless #f 10)" => "10"];
    }

    #[test]
    fn let_lambda() {
        evals!["(let () (+ 10 20))" => "30"];
        evals!["(let ([x 10] [y 20]) (+ x y))" => "30"];
    }

    #[test]
    fn let_star() {
        evals!["(let* ([x 10] [y (* x x)]) (+ x y))" => "110"]
    }

    #[test]
    fn num_compare() {
        evals![
        "(= 1 1)" => "#t",
        "(= 1 1 1 1 1)" => "#t",
        "(= 1 1 2 1 1)" => "#f"
        ];
        evals![
          "(> 5 4)" => "#t",
          "(> 5 4 3 2 1)" => "#t",
          "(> 5 4 3 4 2 1)" => "#f"
        ];
        evals![
          "(>= 2 1)" => "#t",
          "(>= 5 4 3 3 2 1)" => "#t",
          "(>= 5 4 3 4 2 1)" => "#f"
        ];
        evals![
          "(< 1 2)" => "#t",
          "(< 1 2 3 4 5)" => "#t",
          "(< 1 2 1 3 4)" => "#f"
        ];
        evals![
          "(>= 5 4)" => "#t",
          "(>= 5 4 3 3 2 1)" => "#t",
          "(>= 5 4 3 2 3 1)" => "#f"
        ];
        evals![
            "(> 2.0 1)" => "#t",
            "(> 2 1.0)" => "#t"
        ];
    }

    #[test]
    fn num_unary() {
        evals![
            "(zero? 0)" => "#t",
            "(zero? 10)" => "#f",
            "(positive? 10)" => "#t",
            "(positive? 0)" => "#f",
            "(positive? -10)" => "#f",
            "(negative? -10)" => "#t",
            "(negative? 0)" => "#f",
            "(negative? 10)" => "#f",
            "(odd? 3)" => "#t",
            "(odd? 2)" => "#f",
            "(even? 2)" => "#t",
            "(even? 3)" => "#f"
        ];
    }

    #[test]
    fn set() {
        evals!["(define (generator) (let ([x 0]) (lambda () (set! x (+ x 1)) x)))" => "#<void>",
               "(define counter (generator))" => "#<void>",
               "(counter)" => "1",
               "(counter)" => "2",
               "(counter)" => "3"
        ];
    }

    #[test]
    fn set_pair() {
        evals!["(define l '(1 2 3))" => "#<void>",
               "(set-car! (cdr l) 100))" => "#<void>",
               "l" => "(1 100 3)"
        ];
        evals!["(define l '(1 2 3))" => "#<void>",
               "(set-cdr! (cdr (cdr l)) '(4 5 6)))" => "#<void>",
               "l" => "(1 2 3 4 5 6)"
        ];
    }

    #[test]
    fn internal_define() {
        evals!["((lambda (x) (define y 10) (+ x y)) 20)" => "30"];
        fails!["(lambda (x) (define y 10) (+ x y) (define z 10))" => 
            InvalidDefineSyntax("out of context: (define z 10)".into())];
    }

    #[test]
    fn internal_define_is_lexical() {
        evals![
            "(define y 100)" => "#<void>",
            "((lambda (x) (define y 10) (+ x y)) 20)" => "30",
            "y" => "100"
        ];
        evals![
            "(define (y) 100)" => "#<void>",
            "((lambda (x) (define (y) 10) (+ x (y))) 20)" => "30",
            "(y)" => "100"
        ];
    }

    #[test]
    fn vector_and_make_vector() {
        evals![
            "(vector)" => "#()",
            "(vector (+ 10 10) #t)" => "#(20 #t)"
        ];
        evals![
            "(make-vector 0)" => "#()",
            "(make-vector 3)" => "#(0 0 0)",
            "(make-vector 3 '(1 2))" => "#((1 2) (1 2) (1 2))"
        ];
    }

    #[test]
    fn vector_length() {
        evals!["(vector-length #())" => "0",
               "(vector-length #(1 2 3))" => "3"
        ];
        fails!["(vector-length '(1 2 3)))" => 
            InvalidSyntax("(1 2 3) is not a vector".into())];
    }

    #[test]
    fn vector_ref() {
        evals!["(vector-ref #(1 2 3) 0)" => "1",
               "(vector-ref #(1 2 3) 1)" => "2",
               "(vector-ref #(1 2 3) 2)" => "3"
        ];
        fails!["(vector-ref '(1 2 3) 0))" => 
            InvalidSyntax("(1 2 3) is not a vector".into())];
        fails!["(vector-ref #(1 2 3) 10))" => 
            InvalidVectorIndex(10, 3)];
    }

    #[test]
    fn vector_set() {
        evals!["(define v #(1 2 3))" => "#<void>",
               "(vector-set! v 0 42)" => "#<void>",
               "v" => "#(42 2 3)"
        ];
        evals!["(define v #(1 2 3))" => "#<void>",
               "(vector-fill! v 42)" => "#<void>",
               "v" => "#(42 42 42)"
        ];
        fails!["(vector-set! '(1 2 3) 0 0))" =>
            InvalidSyntax("(1 2 3) is not a vector".into())];
        fails!["(vector-fill! '(1 2 3) 0))" =>
            InvalidSyntax("(1 2 3) is not a vector".into())];
        fails!["(vector-set! #(1 2 3) 10 0))" =>
            InvalidVectorIndex(10, 3)];
    }

    #[test]
    fn list_vector_conversions() {
        evals!["(vector->list #(1 2 3))" => "(1 2 3)",
               "(vector->list #())" => "()"
        ];
        evals!["(list->vector '(1 2 3))" => "#(1 2 3)",
               "(list->vector '())" => "#()"
        ];
        fails!["(vector->list '(1 2 3))" =>
            InvalidSyntax("(1 2 3) is not a vector".into())];
        fails!["(list->vector #(1 2 3))" =>
            ExpectedPairButFound("#(1 2 3)".into())];
    }

    #[test]
    fn append() {
        evals!["(append '())" => "()",
               "(append '(1 2 3))" => "(1 2 3)",
               "(append '(1 2 3) '(4 5 6))" => "(1 2 3 4 5 6)",
               "(append '(1 2 3) '() '(4 5 6))" => "(1 2 3 4 5 6)",
               "(append '(1 2 3) '() '(4 5 6 . 7))" => "(1 2 3 4 5 6 . 7)",
               "(append '(1) '(2) '(3))" => "(1 2 3)",
               "(append '(1) '(2) 3)" => "(1 2 . 3)"
        ];
        fails!["(append '(1 2 . 3) '())" => InvalidSyntax("(1 2 . 3) is an improper list".into())]
    }

    #[test]
    fn reverse() {
        evals!["(reverse '())" => "()",
               "(reverse '(1 2 3))" => "(3 2 1)"
        ];
        fails!["(reverse '(1 2 . 3))" => InvalidSyntax("(1 2 . 3) is an improper list".into())]
    }

    #[test]
    fn list_tail() {
        evals!["(list-tail '() 0)" => "()",
                "(list-tail '(1 2 3) 0)" => "(1 2 3)",
                "(list-tail '(1 2 3) 1)" => "(2 3)",
                "(list-tail '(1 2 3) 2)" => "(3)",
                "(list-tail '(1 2 3) 3)" => "()",
                "(list-tail '(1 2 . 3) 2)" => "3"
        ];
        fails![
            "(list-tail '(1 2 3) 4) " => InvalidSyntax("4 is out of range for (1 2 3)".into())
        ];
    }

    #[test]
    fn list_ref() {
        evals!["(list-ref '(1 2 3) 0)" => "1",
                "(list-ref '(1 2 3) 1)" => "2",
                "(list-ref '(1 2 3) 2)" => "3"
        ];
        fails![
            "(list-ref '() 0)" => InvalidSyntax("0 is out of range for ()".into()),
            "(list-ref '(1 2 3) 3)" => InvalidSyntax("3 is out of range for (1 2 3)".into()),
            "(list-ref '(1 2 3) 4)" => InvalidSyntax("4 is out of range for (1 2 3)".into())
        ];
    }

    #[test]
    fn assoc() {
        evals!["(assoc 0 '((0 foo) (1 bar) (2 baz)))" => "(0 foo)",
               "(assoc 2 '((0 foo) (1 bar) (2 baz)))" => "(2 baz)",
               "(assoc 3 '((0 foo) (1 bar) (2 baz)))" => "#f",
               "(assoc '(1 2) '((0 foo) ((1 2) bar) (2 baz)))" => "((1 2) bar)"
        ];
        evals!["(assq 0 '((0 foo) (1 bar) (2 baz)))" => "(0 foo)",
               "(assq 2 '((0 foo) (1 bar) (2 baz)))" => "(2 baz)",
               "(assq 3 '((0 foo) (1 bar) (2 baz)))" => "#f",
               "(assq '(1 2) '((0 foo) ((1 2) bar) (2 baz)))" => "#f"

        ];
        evals!["(assv 0 '((0 foo) (1 bar) (2 baz)))" => "(0 foo)",
               "(assv 2 '((0 foo) (1 bar) (2 baz)))" => "(2 baz)",
               "(assv 3 '((0 foo) (1 bar) (2 baz)))" => "#f",
               "(assv '(1 2) '((0 foo) ((1 2) bar) (2 baz)))" => "#f"
        ];
    }

    #[test]
    fn find_primes() {
        evals![
            r#"
            (define (find-primes n)
                (define (make-sieve n)
                    (define (init-sieve v n)
                        (cond
                            ((zero? n) v)
                            (else (vector-set! v (- n 1) (- n 1)) (init-sieve v (- n 1)))))
                    (init-sieve (make-vector n) n))
                (define (mark-multiples-of v m i)
                    (cond
                        ((>= (* m i) (vector-length v)) v)
                        (else (vector-set! v (* m i) #f) (mark-multiples-of v m (+ i 1)))))
                (define (sieve v i)
                    (cond
                        ((>= i (vector-length v)) v)
                        ((eq? (vector-ref v i) #f) (sieve v (+ i 1)))
                        (else (sieve (mark-multiples-of v i i) (+ i 1)))))
                (define (sieve->list v)
                    (define (sieve->list v i)
                        (cond
                            ((= i (vector-length v)) '())
                            ((eq? (vector-ref v i) #f) (sieve->list v (+ i 1)))
                            (else (cons i (sieve->list v (+ i 1))))))
                    (sieve->list v 0))
                (sieve->list (sieve (make-sieve n) 2)))
            "# => "#<void>",
            "(find-primes 100)" => "(0 1 2 3 5 7 11 13 17 19 23 29 31 37 41 43 47 53 59 61 67 71 73 79 83 89 97)"
        ];
    }

    #[test]
    fn copy_closure() {
        evals![r#"
          (define (copy a b)
             ((lambda (partial) (cons a partial))
             (if (= b 0)
                'eol
                (copy (+ a 1) (- b 1)))))"# => "#<void>",
           "(copy 100 10)" => "(100 101 102 103 104 105 106 107 108 109 110 . eol)"
        ];
    }

    #[test]
    fn apply_makes_environment() {
        evals![
            "(define x 0)" => "#<void>",
            "(let ~ ((i (* 9)))
              (if (< 1 i) (~ (- i 1)))
              (set! x (+ x i)))" => "#<void>",
            "x" => "45"
        ]
    }
}
