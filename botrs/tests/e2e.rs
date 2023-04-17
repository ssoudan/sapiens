use std::rc::Rc;

use botrs::invoke_tool;
use botrs::tools::hue::room::fake::FakeRoomTool;
use botrs::tools::hue::status::fake::FakeStatusTool;
use botrs::tools::{python, Toolbox};
use indoc::indoc;

#[test]
fn test_python() {
    let mut toolbox = Toolbox::default();
    toolbox.add_tool(FakeRoomTool::default());
    toolbox.add_tool(FakeStatusTool::default());
    toolbox.add_advanced_tool(python::PythonTool::default());

    let toolbox = Rc::new(toolbox);

    let input = indoc! {
    r#"```yaml
       command: SandboxedPython
       input:
         code: |
           # Get a list of all rooms and the number of lights in each:
           args = {
               'room_filter': []
           }
           #rooms = tools.Room(room_filter=[])
           rooms = tools.Room(**args)           
          
           num_lights_by_room = {room['name']: len(room['lights']) for room in rooms['rooms']}                      

           # Determine the maximum number of lights in a room:
           max_lights = max(num_lights_by_room.values())                         

           # Get the name of the room with the most lights:
           rooms_with_most_lights = [room for room in num_lights_by_room.keys() if num_lights_by_room[room] == max_lights]
           room_with_most_lights = rooms_with_most_lights[0]
        
           # Get the list of lights in the room with the most lights:
           lights_in_room = tools.LightStatus(light_filter=tools.Room(room_filter=[room_with_most_lights])['rooms'][0]['lights'])['lights']
           light_names = ', '.join([light['name'] for light in lights_in_room])
        
           print("The following lights are in the room with the most lights:")
           print(light_names)
       ```
    "#};

    let res = invoke_tool(toolbox, input).unwrap();

    assert_eq!(
        res,
        "stdout: |\n  The following lights are in the room with the most lights:\n  Bed, Closet\nstderr: ''\n"
    );
}
