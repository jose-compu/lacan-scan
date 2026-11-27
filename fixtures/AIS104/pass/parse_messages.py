def openai_chat(messages):
    system_prompt, conversation_messages, err = _parse_chat_messages(messages)
    return system_prompt, conversation_messages