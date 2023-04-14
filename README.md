# Botrs 

Fun with chatGPT API, [llm-chain](https://github.com/sobelio/llm-chain) and 
[huelib (fork)](https://github.com/ssoudan/huelib-rs/).

## Disclaimer

This will probably try to take over the world and if it doesn't succeed, 
empty your bank account and kill all the remaining polar bears trying to do so.

## Usage

`.env` file with: 
```
OPENAI_API_KEY=...
HUE_USERNAME=...
HUE_BRIDGE_IP=...
```
Look at `src/main.rs` if you don't already have these.

# Example of 'successful' run

```
$ cargo run
```
Compiling botrs v0.1.0 (ssoudan/botrs)

### Warm up

[... half a decade of compilation later ...]
<pre><code>
    Finished dev [unoptimized + debuginfo] target(s) in 4.18s
     Running `target/debug/botrs`
</code></pre>

Here start the <strike>show</strike> warm up prompting:

The WORLD:

<pre><code>
You are botGPT, large language model assisting the WORLD. Use available tools to answer the question as best as you can.
You will proceed in a OODA loop made of the following steps:
- Observations: What do you know? What is your source? What don't you know? You might want to note down important information for later like relevant past Action results.
- Orientation: Plan the intermediate objectives along the path to answer the original question. Make a list of current objectives.
- Decision: Choose what to do first to answer the question. Why? What are the pros and cons of this decision?
- Action: Take a single Action consisting of exactly one tool invocation. The available Tools listed below. Use ConcludeTool when you have the final answer to the original question.

# Notes:
- Template expansion is not supported in Action. If you need to pass information from on action to another, you have to pass them manually.
- Use ConcludeTool with your conclusion (as text) once you have the final answer to the original question.
- There are no APIs available to you. You have to use the Tools.

# Format of your response

Please use the following format for your response - no need to be verbose:
====================
## Observations:
- ...
## Orientation:
- ...
## Decision:
- ...
## The ONLY Action: <Do not give multiple command. Only one per response.>
```yaml
command: <ToolName>
input:
  <... using the `input_format` for the Tool ...>
```
====================

# The following are the ONLY Tools you can use for your Actions:
- name: RoomTool
  description: A tool to use that the source of truth for the Lights of a Room.
  description_context: Use this to fetch the Lights of Rooms.
  input_format:
    room_filter: 'The list of Room names (<string>) to get the lights for, e.g. `room_filter: ["Bedroom"]`. Use `[]` to get all Rooms.'
- name: ConcludeTool
  description: A tool to terminate a task with a conclusion.
  description_context: Use this to terminate a task when it is complete.
  input_format:
    conclusion: The final textual answer for this task. No string interpolation supported, only plain text. MANDATORY.
- name: SandboxedPythonTool
  description: 'A tool that executes sandboxed Python code. Only stdout and stderr are captured and made available. '
  description_context: Use this to transform data. This is not a tool to retrieve information. Except `print()`, no interactions with the world. No input. No `import`. No library. No API access. Just plain Python. import|open|exec|eval|__import__ are forbidden.
  input_format:
    code: 'The Python code to execute. For example: `data = [...]; <...>; output = <...> ; print(output)`. MANDATORY'

</code></pre>

botGPT:

<pre><code>
Understood.
</code></pre>   

WORLD:

<pre><code>
## Original question: Sort in ascending order: [2, 3, 1, 4, 5].
</code></pre>

botGPT:

<pre><code>
## Observations:
- The given list to sort is [2, 3, 1, 4, 5].
- I need to sort this list in ascending order.
## Orientation:
- SandboxedPythonTool can be used to sort the list.
## Decision:
- We can use the sorted() function of Python to sort the list.
## The ONLY Action:
```yaml
command: SandboxedPythonTool
input:
  code: |
    lst = [2, 3, 1, 4, 5]
    sorted_list = sorted(lst)
    print(f"The sorted list is {sorted_list}")
```
</code></pre>

WORLD:

<pre><code>
# Action result:
```yaml
status: 0
stdout: |
  The sorted list is [1, 2, 3, 4, 5]
stderr: ''
```
# Your turn
Original question: Sort [2, 3, 1, 4, 5]
Observations, Orientation, Decision, The ONLY Action?
</code></pre>

botGPT:

<pre><code>
# Observations:
- We needed to sort the list in ascending order.
- We have the result of the Action.
- We have the sorted list: [1, 2, 3, 4, 5].
# Orientation:
- I know the answer to the original question.
# Decision:
- Use the ConcludeTool to terminate the task with the sorted list.
# The ONLY Action:
```yaml
command: ConcludeTool
input:
  conclusion: |
    The ascending sorted list is [1, 2, 3, 4, 5].
```
</code></pre>

This was the warm up.

### Now the real thing

WORLD:
<pre><code>
# Your turn
Original question: What are the names of the rooms?
Observations, Orientation, Decision, The ONLY Action?
</code></pre>

botGPT: 

<pre><code>
# Observations:
- We need to know the names of the rooms.
# Orientation:
- We can use RoomTool to fetch the names of rooms.
# Decision:
- Use RoomTool to get the names of rooms available.
# The ONLY Action:
```yaml
command: RoomTool
input:
  room_filter: []
```
</code></pre>

WORLD:

<pre><code>
# Action result:
```yaml
rooms:
- name: Living
  lights:
  - '4'
  - '5'
  - '6'
  - '21'
- name: Room
  lights:
  - '32'
  - '3'
  - '1'
  - '9'
  - '10'
  - '11'
  - '19'
- name: Office
  lights:
  - '13'
  - '27'
  - '2'
  - '16'
  - '26'
  - '25'
  - '24'
  - '15'
  - '17'
  - '18'
  - '20'
- name: Kitchen
  lights:
  - '31'
  - '30'
  - '22'
  - '23'
- name: Hallway
  lights:
  - '33'
  - '28'
```
# Your turn
Original question: What are the names of the rooms?
Observations, Orientation, Decision, The ONLY Action?
</code></pre>

botGPT:

<pre><code>
# Observations:
- The RoomTool returned a list of rooms along with their light fixtures.
- The room names are: Living, Room, Office, Kitchen and Hallway.
# Orientation:
- I have the answer to the original question.
# Decision:
- Use ConcludeTool to terminate the current task with the room names.
# The ONLY Action:
```yaml
command: ConcludeTool
input:
  conclusion: |
    The names of the rooms are Living, Room, Office, Kitchen and Hallway.
```
</code></pre>

Finally, ConcludeTool is used to terminate the task with the answer:
<pre><code>
And the conclusion is: The names of the rooms are Living, Room, Office, Kitchen and Hallway.
</code></pre>

Not bad. Not all the sessions are like this one.