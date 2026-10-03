# QUESTION ONE

- **src/scanner.rs:238**
- The scanner would would scan the number, if and only if self.peek() is '.' and self.peek_next() is a valid number (is_ascii_digit()) it continues to add the numbers to the nunber_holder vector, else in the case of 5. where self.peek_next is not a valid number, it would throw the error Character is not part of any token.
- Because there is no DOT token in kobo, assuming there was then the scanner would just tokenize 5 and . separately

# QUESTION TWO

- **src/scanner.rs:67, src/scanner.rs:69, src/scanner.rs:148, src/scanner.rs:195**
- The EOF token will carry the last line with a valid token or invalid token (that isnt just comments or for instance any other token that it has been designed to completely ignore), literally any line that has a character
- Section 6.1 requests for that because those lines or completely empty lines (in the absence of valid patterns like inside a strings ") are not grouped as meaningful tokens, and if they were it would be a waste of space in the tokens vector

# QUESTION THREE

- valid\identifiers.kobo, it was fixed on **src/scanner.rs:268**
- For all continuous pattern functions like strings, numbers and identifiers, for identifiers specifically on **src/scanner.rs:266**, I used a vector (**identifier_holder**) to store their characters before concatenating and checking if the concatenated characters matched a keyword however I noticed through println debugs that were used to display what exactly was inside of identifier_holder that because the scanner advances from pattern_beginning at **src/scanner.rs:179** in **fn scan_token()**, when the scanner sees the start of the pattern of an identifier, because the scanner had advanced passed the beginning of the identifier before identifier_holder was even initialized, identifier_holder started appending 1 character after the beginning of the identifier and thus all keywords showed up as identifiers in the test (e.g the word print in the debug showed that the identifier_holders concat, holder_slice at **src/scanner.rs:282** was "rint" and since that didnt match any keyword it was seen as an identifier). To fix it I ensured to append the previous char at the index that pattern_beginning advanced before entering **fn identifier()** and it fixed the bug, the identifiers test passed after that fix.

- **valid\identifiers.kobo bug git hash** - edf09997ca7102dc53d324412568cd2d52180f58
- **valid\identifiers.kobo fix git hash** - 537b6eb7b98f6da27634e596a684657334f12562
