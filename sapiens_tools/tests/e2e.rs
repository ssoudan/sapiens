//! Tests
#[cfg(feature = "hue")]
mod hue_test {
    use indoc::indoc;
    use insta::{assert_display_snapshot, assert_yaml_snapshot};
    use pyo3::PyResult;
    use sapiens::tools::toolbox::{invoke_tool, InvokeResult, Toolbox};
    use sapiens_tools::conclude::ConcludeTool;
    use sapiens_tools::hue::room::fake::FakeRoomTool;
    use sapiens_tools::hue::status::fake::FakeStatusTool;
    use sapiens_tools::python::PythonTool;

    #[pyo3_asyncio::tokio::test]
    async fn test_python_hue() -> PyResult<()> {
        let data = indoc! {r#"
    # Action
    ```yaml
    tool_name: SandboxedPython
    parameters:
      code: |
        room_name = "Bedroom"
    
        import json
    
        # Get the list of room objects with names and Lights ID lists.
        rooms = tools.Room(room_filter=[]).get('rooms')
    
        # Find the relevant room and get only its lights IDs.
        room_lights = next(room for room in rooms if room_name in room.get('name'))
        room_lights_ids = room_lights.get('lights')
                        
        # Get the statuses of Lights in the Bedroom.
        Bedroom_lights_status = tools.LightStatus(light_filter=room_lights_ids).get('lights')
    
        # Conclude the task with the result.
        tools.Conclude(
            original_question="what is the status of the lights in the Bedroom?",
            conclusion=f"The light(s) status in {room_name} is {json.dumps(Bedroom_lights_status)}"
        )
    ```
    "#};

        let mut toolbox = Toolbox::default();
        toolbox.add_advanced_tool(PythonTool::default()).await;
        toolbox.add_terminal_tool(ConcludeTool::default()).await;
        toolbox.add_tool(FakeRoomTool::default()).await;
        toolbox.add_tool(FakeStatusTool::default()).await;

        let res = invoke_tool(toolbox.clone(), data).await;

        match res {
            InvokeResult::Success {
                tool_name, result, ..
            } => {
                assert_eq!(tool_name, "SandboxedPython");
                assert_display_snapshot!(result);
            }
            _ => panic!("Unexpected result: {:?}", res),
        }

        // collect the conclusion
        let termination_messages = toolbox.termination_messages().await;

        assert_yaml_snapshot!(termination_messages);

        Ok(())
    }
}

#[cfg(feature = "arxiv")]
mod arxiv {
    use indoc::indoc;
    use insta::{assert_display_snapshot, assert_yaml_snapshot};
    use pyo3::PyResult;
    use sapiens::tools::toolbox::{invoke_tool, InvokeResult, Toolbox};
    use sapiens_tools::arxiv::ArxivTool;
    use sapiens_tools::conclude::ConcludeTool;
    use sapiens_tools::python::PythonTool;

    #[pyo3_asyncio::tokio::test]
    async fn test_python_arxiv() -> PyResult<()> {
        let mut toolbox = Toolbox::default();
        toolbox.add_tool(ArxivTool::new().await).await;
        toolbox.add_advanced_tool(PythonTool::default()).await;

        let data = indoc! {r#"```yaml
   tool_name: SandboxedPython
   parameters:
     code: |           
           import datetime               
           search_query = f'cat:cs.AI'
           input = {
               'search_query': search_query,
               'start': 0,
               'max_results': 10,
               'sort_by': 'lastUpdatedDate',
               'sort_order': 'ascending'
           }
           arxiv_output = tools.Arxiv(**input)
           papers = arxiv_output['result']               
           conclusion = f"Yes, there are {len(papers)} published papers of AI category on ArXiv this week." if papers else "No, there are no published papers of AI category on ArXiv this week."
           print(conclusion)
   ```
"#};

        let res = invoke_tool(toolbox, data).await;

        match res {
            InvokeResult::Success {
                tool_name, result, ..
            } => {
                assert_eq!(tool_name, "SandboxedPython");
                assert_display_snapshot!(result);
            }
            _ => panic!("Unexpected result: {:?}", res),
        }

        Ok(())
    }

    #[pyo3_asyncio::tokio::test]
    async fn test_python_arxiv_2() -> PyResult<()> {
        let mut toolbox = Toolbox::default();
        toolbox.add_tool(ArxivTool::new().await).await;
        toolbox.add_advanced_tool(PythonTool::default()).await;

        let data = indoc! {r#"```yaml
   tool_name: SandboxedPython
   parameters:
     code: |           
           import datetime               
           search_query = f'cat:cs.AI'               
           arxiv_output = tools.Arxiv(search_query=search_query, start=0, max_results=10, sort_by='lastUpdatedDate', sort_order='ascending')               
           papers = arxiv_output['result']               
           conclusion = f"Yes, there are {len(papers)} published papers of AI category on ArXiv this week." if papers else "No, there are no published papers of AI category on ArXiv this week."
           print(conclusion)
   ```
"#};

        let res = invoke_tool(toolbox, data).await;

        match res {
            InvokeResult::Success {
                tool_name, result, ..
            } => {
                assert_eq!(tool_name, "SandboxedPython");
                assert_display_snapshot!(result);
            }
            _ => panic!("Unexpected result: {:?}", res),
        }

        Ok(())
    }

    #[pyo3_asyncio::tokio::test]
    async fn test_python_arxiv_3() -> PyResult<()> {
        let mut toolbox = Toolbox::default();
        toolbox.add_tool(ArxivTool::new().await).await;
        toolbox.add_advanced_tool(PythonTool::default()).await;

        let data = indoc! {r#"```yaml
   tool_name: SandboxedPython
   parameters:
     code: |
        latest_papers = tools.Arxiv(
          search_query="cat:cs.DB",
          max_results=4,
          show_summary=True,
          show_pdf_url=True
          )["result"]           
        print(latest_papers)
   ```
"#};

        let res = invoke_tool(toolbox, data).await;

        match res {
            InvokeResult::Success {
                tool_name, result, ..
            } => {
                assert_eq!(tool_name, "SandboxedPython");
                assert_display_snapshot!(result);
            }
            _ => panic!("Unexpected result: {:?}", res),
        }

        Ok(())
    }

    #[pyo3_asyncio::tokio::test]
    async fn test_python_arxiv_4() -> PyResult<()> {
        let mut toolbox = Toolbox::default();
        toolbox.add_tool(ArxivTool::new().await).await;
        toolbox.add_terminal_tool(ConcludeTool::default()).await;
        toolbox.add_advanced_tool(PythonTool::default()).await;

        let data = indoc! {r#"```yaml
    tool_name: SandboxedPython
    parameters:
      code: |
        latest_papers = tools.Arxiv(
          search_query="cat:cs.DB",
          max_results=4,
          show_summary=True,
          show_pdf_url=True,
          sort_order="ascending",
          )["result"]
    
        summaries = []
        for paper in latest_papers:
            summary = paper["summary"]
            summaries.append(summary)
    
        task_conclusion = "\n\n\n".join([f"Title: {paper['title']}\nSummary: {summary}\nPDF Url: {paper['pdf_url']}"
                                        for paper, summary in zip(latest_papers, summaries)])
    
        tools.Conclude(
                original_question="What are the 4 latest papers on database published on arXiv? What are they about?",
                conclusion=task_conclusion)
   ```
"#};

        let res = invoke_tool(toolbox.clone(), data).await;

        match res {
            InvokeResult::Success {
                tool_name, result, ..
            } => {
                assert_eq!(tool_name, "SandboxedPython");
                assert_display_snapshot!(result);
            }
            _ => panic!("Unexpected result: {:?}", res),
        }

        let termination_messages = toolbox.termination_messages().await;
        let done = !termination_messages.is_empty();
        assert!(done);

        assert_yaml_snapshot!(termination_messages);

        Ok(())
    }
}

mod python;

#[cfg(all(feature = "search", not(feature = "disable-test-dependabot")))]
mod search {
    use indoc::indoc;
    use insta::assert_display_snapshot;
    use pyo3::PyResult;
    use sapiens::tools::toolbox::{invoke_tool, InvokeResult, Toolbox};
    use sapiens_tools::python::PythonTool;
    use sapiens_tools::search::SearchTool;

    #[pyo3_asyncio::tokio::test]
    async fn test_python_search() -> PyResult<()> {
        let mut toolbox = Toolbox::default();
        toolbox.add_tool(SearchTool::default()).await;
        toolbox.add_advanced_tool(PythonTool::default()).await;

        let data = indoc! {r#"```yaml
   tool_name: SandboxedPython
   parameters:
     code: |                
         input = {"q": "rust" }     
         output = tools.Search(**input)                                     
         print(len(output['results']))
   ```
"#};

        let res = invoke_tool(toolbox, data).await;

        match res {
            InvokeResult::Success {
                tool_name, result, ..
            } => {
                assert_eq!(tool_name, "SandboxedPython");
                assert_display_snapshot!(result);
            }
            _ => panic!("Unexpected result: {:?}", res),
        }

        Ok(())
    }
}

#[pyo3_asyncio::tokio::main(flavor = "multi_thread")]
async fn main() -> pyo3::PyResult<()> {
    let _ = dotenvy::dotenv();

    pyo3_asyncio::testing::main().await
}
