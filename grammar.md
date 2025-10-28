# Proper Language Definitions

## Grammar

Program ::= Block
Block ::= "{" { Statement } "}"
Statement ::= IfStatement
            | WhileStatement
            | ForStatement
            | FunctionDefinition
            | ReturnStatement
            | MatchStatement
            | LetStatement
            | BreakStatement
            | ContinueStatement
            | ExpressionStatement

IfStatement ::= "if" Expression Block \[ "else" ( Block | IfStatement ) \]

WhileStatement ::= "while" Expression Block

ForStatement ::= "for" IDENTIFIER "in" Expression Block

BreakStatement ::= "break" ";"
ContinueStatement ::= "continue" ";"

FunctionDefinition ::= "fn" IDENTIFIER "(" ParameterList ")" Block

ReturnStatement ::= "return" \[ Expression \] ";"

MatchStatement ::= "match" Expression "{" \[ MatchArm { "," MatchArm } \[ "," \] \] "}" 
MatchArm ::= Pattern "=>" ( Expression | Block )
Pattern ::= LITERAL | IDENTIFIER | "_"

LetStatement ::= "let" IDENTIFIER \[ "=" Expression \] ";"

ParameterList ::= IDENTIFIER { "," IDENTIFIER }

ExpressionStatement ::= Expression ";"

Expression ::= LogicalOrExpression
LogicalExpression ::= EqualityExpression { ( "||" | "&&" ) EqualityExpression }
EqualityExpression ::= RelationalExpression { ( "==" | "!=" ) RelationalExpression }
RelationalExpression ::= AdditiveExpression { ( "<" | ">" | "<=" | ">=" ) AdditiveExpression }
AdditiveExpression ::= MultiplicativeExpression { ( "+" | "-" ) MultiplicativeExpression }
MultiplicativeExpression ::= UnaryExpression { ( "*" | "/" | "%" ) UnaryExpression }
UnaryExpression ::= \[ ( "!" | "-" | "+" ) \] PrimaryExpression
PrimaryExpression ::= Atom { Postfix }
Atom ::= LITERAL
        | IDENTIFIER
        | "(" Expression ")"
        | Array
        | Object

Postfix ::= FunctionCall
          | IndexAccess
          | FieldAccess

FunctionCall ::= "(" [ ArgumentList ] "")"
IndexAccess ::= "\[" Expression "\]"
FieldAccess ::= "." IDENTIFIER

ArgumentList ::= Expression { "," Expression }

Array ::= "\[" \[ Expression, { "," Expression } \[ "," \] \] "\]"
Object ::= "{" \[ ObjectEntry { "," ObjectEntry } \[ "," \] \] "}"
ObjectEntry ::= IDENTIFIER ":" Expression

## Tokens

### Character Definitions

BIN_DIGIT ::= "0" | "1"
HEX_DIGIT ::= DIGIT | "A".."F" | "a".."f"
DIGIT ::= "0".."9"

### Escape & String Components

ESCAPE_CHAR     ::= "n" | "r" | "t" | "0" | "'" | "\"" | "\\"
ESCAPE_SEQUENCE ::= "\\" ESCAPE_CHAR
LETTER ::= "a".."z" | "A".."Z"
CHAR_CONTENT ::= ANY_CHARACTER_EXCEPT_QUOTE_OR_BACKSLASH | ESCAPE_SEQUENCE
STRING ::= """ { CHAR_CONTENT } """

#### Literals

HEX_LITERAL ::= "0" ( "x" | "X" ) HEX_DIGIT { HEX_DIGIT }
BIN_LITERAL ::= "0" ( "b" | "B" ) BIN_DIGIT { BIN_DIGIT }
INTEGER ::= DIGIT { DIGIT }
FLOAT ::= DIGIT { DIGIT } "." { DIGIT }
        | "." DIGIT { DIGIT }
BOOLEAN ::= "true" | "false"
LITERAL ::= HEX_LITERAL | BIN_LITERAL | INTEGER | FLOAT | BOOLEAN

#### Identifiers & Keywords

IDENTIFIER ::= LETTER { LETTER | DIGIT }
KEYWORD ::= "if" | "else" | "while" | "for" | "fn" | "let" | "return" | "match" | "continue" | "break"

#### Whitespace & Comments

WHITESPACE ::= " " | "\t" | "\n" | "\r" 
COMMENT ::= "//" { ANY_CHARACTER_EXCEPT_NEWLINE } "\n"
          | "/*" { ANY_CHARACTER } "\*/"

#### Arithmetic Operators

PLUS ::= "+"
MINUS ::= "-"
STAR ::= "*"
SLASH ::= "/"
PERCENT ::= "%"

#### Comparison Operators

EQ ::= "=="
NEQ ::= "!="
LT ::= "<"
LTE ::= "<="
GT ::= ">"
GTE ::= ">="

#### Logical Operators

AND ::= "&&"
OR ::= "||"
BANG ::= "!"

#### Bitwise Operators

AMP         ::= "&"
PIPE        ::= "|"
CARET       ::= "^"
LSHIFT      ::= "<<"
RSHIFT      ::= ">>"
LSHIFT_ASSIGN ::= "<<="
RSHIFT_ASSIGN ::= ">>="
AND_ASSIGN ::= "&="
PIPE_ASSIGN ::= "|="
CARET_ASSIGN ::= "^="

#### Assignment Operators

ASSIGN ::= "="
PLUS_ASSIGN ::= "+="
MINUS_ASSIGN ::= "-="
STAR_ASSIGN ::= "*="
SLASH_ASSIGN ::= "/="
PERCENT_ASSIGN ::= "%="

#### Punctuation

LPAREN      ::= "("
RPAREN      ::= ")"
LBRACE      ::= "{"
RBRACE      ::= "}"
LBRACKET    ::= "\["
RBRACKET    ::= "\]"
COMMA       ::= ","
SEMICOLON   ::= ";"
COLON       ::= ":"
DOT         ::= "."
