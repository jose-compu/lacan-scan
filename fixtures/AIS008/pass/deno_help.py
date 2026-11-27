def require_deno():
    raise FileNotFoundError(
        "Deno executable not found. Install with:\n"
        "> curl -fsSL https://deno.land/install.sh | sh\n"
    )