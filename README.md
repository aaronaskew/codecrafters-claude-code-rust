[![progress-banner](https://backend.codecrafters.io/progress/claude-code/77c05af9-671d-49e2-9e2f-adfef4752765)](https://app.codecrafters.io/users/aaronaskew?r=2qF)

This is a starting point for Rust solutions to the
["Build Your own Claude Code" Challenge](https://codecrafters.io/challenges/claude-code).

Claude Code is an AI coding assistant that uses Large Language Models (LLMs) to
understand code and perform actions through tool calls. In this challenge,
you'll build your own Claude Code from scratch by implementing an LLM-powered
coding assistant.

Along the way you'll learn about HTTP RESTful APIs, OpenAI-compatible tool
calling, agent loop, and how to integrate multiple tools into an AI assistant.

**Note**: If you're viewing this repo on GitHub, head over to
[codecrafters.io](https://codecrafters.io) to try the challenge.

# Passing the first stage

The entry point for your `claude-code` implementation is in `src/main.rs`. Study
and uncomment the relevant code, and submit to pass the first stage:

```sh
codecrafters submit
```

# Stage 2 & beyond

Note: This section is for stages 2 and beyond.

1. Ensure you have `cargo (1.95)` installed locally.
2. Run `./your_program.sh` to run your program, which is implemented in
   `src/main.rs`. This command compiles your Rust project, so it might be slow
   the first time you run it. Subsequent runs will be fast.
3. Run `codecrafters submit` to submit your solution to CodeCrafters. Test
   output will be streamed to your terminal.

```python
messages = [{ role: "user", content: prompt }]

loop:
    response = call_api(messages)
    append response message to messages

    if response has no tool_calls:
        print response.content
        exit

    for each tool_call in response.tool_calls:
        result = execute_tool(tool_call)
        append {
            role: "tool",
            tool_call_id: tool_call.id,
            content: result
        } to messages
```

Tool Result Message

```json

{
  "role": "tool",
  "tool_call_id": "call_abc123",
  "content": "# My Project\n\nChemical expiry period: 6 months"
}

```
