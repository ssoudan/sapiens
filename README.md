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

[... half a decade of compilation later ...]
<pre><code>
    Finished dev [unoptimized + debuginfo] target(s) in 4.18s
     Running `target/debug/botrs`
</code></pre>

Here start the <strike>show</strike> warm up prompting:

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

# The exchange between you (botGPT) and the WORLD will look like this - Note YAML is only used for the Action:
--------
[WORLD]: Question: ...
[botGPT]:
## Observations:
- ...
## Orientation:
- ...
## Decision:
- ...
## The ONLY Action:
```yaml
command: <ToolName>
input:
  <... using the `input_format` for the Tool ...>
```
[WORLD]: Action result:
```yaml
...
```
Original question: ...
Observations, Orientation, Decision, The Action?
[botGPT]:
## Observations:
- ...
## Orientation:
- ...
## Decision:
- ...
## The ONLY Action:
```yaml
command: <ToolName>
input:
  <... using the `input_format` for the Tool ...>
```
--------

Your response only needs to include the Observations, Orientation, Decision and Action. The rest will be filled in automatically.

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

---
# Your turn
Original question: How many lights are in each room?
Observations, Orientation, Decision, The ONLY Action?
</code></pre>

First response from botGPT:
<pre><code>
## Observations:
- We need to determine the number of lights in each room.
- We have a tool called RoomTool that can fetch the lights of each room.

## Orientation:
- The first step is to fetch the list of rooms.
- We will then use the RoomTool to fetch the lights of each room.
- Finally, we will count the number of lights for each room.

## Decision:
- We will use the RoomTool to fetch the list of rooms.
- We will then iterate over each room and use RoomTool to fetch the list of lights for that room.
- Finally, we will count the number of lights for each room and output the result.

## The ONLY Action:
```yaml
command: RoomTool
input:
  room_filter: []
```
</code></pre>

Message to botGPT:

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
- name: Kitchen
  lights:
  - '31'
  - '30'
  - '22'
  - '23'
- name: Room
  lights:
  - '3'
  - '1'
  - '9'
  - '10'
  - '11'
  - '19'
- name: Hallway
  lights:
  - '33'
  - '28'
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
```
# Your turn
Original question: How many lights are in each room?
Observations, Orientation, Decision, The ONLY Action?
</code></pre>

Second response from botGPT:

<pre><code>
## Observations:
- We have the list of rooms with the corresponding list of lights in each room.

## Orientation:
- We need to count the number of lights in each room from the list fetched by the RoomTool.

## Decision:
- We will iterate over each room fetched by the RoomTool.
- For each room, we will count the number of lights in the room.
- We will add the count to a dictionary with the room's name as the key.

## The ONLY Action:
```yaml
command: SandboxedPythonTool
input:
  code: |
    data = [{"name": "Living", "lights": [4, 5, 6, 21]},
            {"name": "Kitchen", "lights": [31, 30, 22, 23]},
            {"name": "Room", "lights": [3, 1, 9, 10, 11, 19]},
            {"name": "Hallway", "lights": [33, 28]},
            {"name": "Office", "lights": [13, 27, 2, 16, 26, 25, 24, 15, 17, 18, 20]}]

    light_count = {}
    for room in data:
        light_count[room["name"]] = len(room["lights"])

    print(light_count)
```
</code></pre>

Message to botGPT:

<pre><code>
# Action result:
```yaml
status: 0
stdout: |
  {'Living': 4, 'Kitchen': 4, 'Room': 6, 'Hallway': 2, 'Office': 11}
stderr: ''
```
# Your turn
Original question: How many lights are in each room?
Observations, Orientation, Decision, The ONLY Action?
</code></pre>

New message from botGPT:

<pre><code>
## Observations:
- We have a Python code that iterates over each room fetched by the RoomTool and counts the number of lights in the room.
- We have the list of rooms with the corresponding counts for the number of lights in each room.

## Orientation:
- We need to output the number of lights for each room.

## Decision:
- We will format the output string with the number of lights for each room.

## The ONLY Action:
```yaml
command: SandboxedPythonTool
input:
  code: |
    data = [{"name": "Living", "lights": [4, 5, 6, 21]},
            {"name": "Kitchen", "lights": [31, 30, 22, 23]},
            {"name": "Room", "lights": [3, 1, 9, 10, 11, 19]},
            {"name": "Hallway", "lights": [33, 28]},
            {"name": "Office", "lights": [13, 27, 2, 16, 26, 25, 24, 15, 17, 18, 20]}]

    light_count = {}
    for room in data:
        light_count[room["name"]] = len(room["lights"])

    output_str = ""
    for room_name, count in light_count.items():
        output_str += f"{room_name}: {count}\n"

    print(output_str)
```
</code></pre>

Message to botGPT:
<pre><code>
# Action result:
```yaml
status: 0
stdout: |+
  Living: 4
  Kitchen: 4
  Room: 6
  Hallway: 2
  Office: 11

stderr: ''
```
# Your turn
Original question: How many lights are in each room?
Observations, Orientation, Decision, The ONLY Action?
</code></pre>

New message from botGPT:

<pre><code>
## Observations:
- We have the number of lights for each room output as a formatted string.

## Orientation:
- The original question has been answered.

## Decision:
- We will use ConcludeTool to terminate the task with the final answer to the original question.

## The ONLY Action:
```yaml
command: ConcludeTool
input:
  conclusion: "There are 4 lights in the Living room, 4 lights in the Kitchen, 6 lights in the Room, 2 lights in the Hallway, and 11 lights in the Office."
```
</code></pre>

The 'ConcludeTool' action has been called and the chain terminated:

<pre><code>
And the conclusion is: There are 4 lights in the Living room, 4 lights in the Kitchen, 6 lights in the Room, 2 lights in the Hallway, and 11 lights in the Office.
</code></pre>

Not bad. Not all the sessions are like this one.