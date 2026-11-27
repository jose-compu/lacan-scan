# Mistral instruct format uses [INST] tokens for counting.
def token_len(s: str) -> int:
    return len(s)