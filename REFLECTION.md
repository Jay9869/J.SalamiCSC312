# QUESTION ONE

- **src/parser.rs:273**
- The if self.matches(&[TokenType::Else]) {} is where the dangling else is resolved
- The previous decision is done at let then_branch = Box::new(self.statement()?); at **src/parser.rs:270** which is where the recursive descent functions and the parser resolves the then branch as a full statement first meaning that if the statement starts with another if, the inner if consumes its own else before the outer parser can which is why the nearest unmatched if owns the else instead of the grammar
- it printed (if false (if true (print "a") (print "b"))), when the else is bound to the outer if it prints (if false (if true (print "a")) (print "b"))

# QUESTION TWO

- **src/parser.rs:348**
- The match left {} is where the assignment check happens.
- It cannot happen before the left side because a valid assignment target and a non assignable expression can begin with the same tokens so the parser will not be able to know whether = is allowed until it has parsed the left side. That is why 1 = 2; prints [line 1] Error at '=': Cannot assign to this expression.

# QUESTION THREE

- **src/parser.rs:610** and it is called in **src/parser.rs:22** when a declaration fails
- The function stops after a ; token (**src/parser.rs:618**) because that is the end of a complete statement meaning it acts as the restarting point for other statements to be done. It also stops when the next token is fun, var, if, while, print or return because those can only begin on a declaration or statement. It will always step over one token first so the parser progresses
- **tests/phase-2/invalid/expect_expression.kobo** (print 1 + \* 2;) (note it is print 1 + asterisk 2, the md file changed the asterisk to that), when synchronise() is called the output is [line 1] Error at '\*': Expect expression. but if the call to it is deleted, the parser does not reset to the next statement boundary so it stays on the bad token and then fails on it again over and over again so it never finishes and gives no output

- Bugged code commit hash - defb178f97aceeee7f97fcc5ecd1cb32302314c7
- Corrected code commit hash -
  b828a566772df5f66f91b7e809414d5cc60e73bb

- In the bugged synchronise, the loop only stopped after a semicolon and had no keyword check, the only stop was if self.previous().kind == TokenType::Semicolon { return; }. I found it with a custom test that contained var 123 print +; and expects two errors [line 1] Error at '123': Expect variable name. and [line 1] Error at '+': Expect expression. The old code printed only the first one because after the bad token it skipped straight through print +; up to the semicolon and it never stopped at print. The edge case is a statement keyword that is coming directly after the error token and before any semicolon so in the corrected code, I added a match statement (match self.peek().kind { TokenType::Fun | TokenType::Var | TokenType::If | TokenType::While | TokenType::Print | TokenType::Return => return, \_ => {} }) for the other keywords so it can stop there and the test will give both errors
