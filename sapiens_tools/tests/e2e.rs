//! Tests

#[pyo3_asyncio::tokio::main(flavor = "multi_thread")]
async fn main() -> pyo3::PyResult<()> {
    pyo3_asyncio::testing::main().await
}

mod tests {
    use indoc::indoc;
    use insta::assert_display_snapshot;
    use pyo3::PyResult;
    use sapiens::tools::{invoke_tool, Toolbox};
    use sapiens_tools::conclude::ConcludeTool;
    use sapiens_tools::dummy::DummyTool;
    use sapiens_tools::python::PythonTool;

    #[pyo3_asyncio::tokio::test]
    async fn test_tool_invocation() -> PyResult<()> {
        let data = indoc! {r#"
        # Action
        ```yaml        
        command: SandboxedPython
        input:
            code: |
                print("Hello world!")          
        ```
        "#};

        let mut toolbox = Toolbox::default();
        toolbox.add_advanced_tool(PythonTool::default()).await;

        let (tool_name, res) = invoke_tool(toolbox, data).await;
        assert_eq!(tool_name, "SandboxedPython");
        let output = res.unwrap();
        assert_eq!(output, "stdout: |\n  Hello world!\nstderr: ''\n");

        Ok(())
    }

    #[pyo3_asyncio::tokio::test]
    async fn test_tool_simple_invocation() -> PyResult<()> {
        let data = indoc! {r#"
        # Action
        ```yaml        
        command: Conclude
        input:
            original_question: |
                print("Hello world!")
            conclusion: |
                Hello world!          
        ```
        "#};

        let mut toolbox = Toolbox::default();
        toolbox.add_advanced_tool(PythonTool::default()).await;
        toolbox.add_tool(ConcludeTool::default()).await;

        let (tool_name, res) = invoke_tool(toolbox, data).await;
        assert_eq!(tool_name, "Conclude");

        let output = res.unwrap();
        assert_display_snapshot!(output);

        Ok(())
    }

    #[pyo3_asyncio::tokio::test]
    async fn test_tool_invocation_in_python() -> PyResult<()> {
        let data = indoc! {r#"
        # Action
        ```yaml        
        command: SandboxedPython
        input:
            code: |
                print("Hello world!")
                rooms = toolbox.invoke("Dummy", {"blah": "blah"})
                print(rooms)
                rooms = tools.Dummy(blah="blah")
                print(rooms)          
        ```
        "#};

        let mut toolbox = Toolbox::default();
        toolbox.add_advanced_tool(PythonTool::default()).await;
        toolbox.add_tool(DummyTool::default()).await;

        let (tool_name, res) = invoke_tool(toolbox, data).await;
        assert_eq!(tool_name, "SandboxedPython");
        let output = res.unwrap();
        assert_display_snapshot!(output);

        Ok(())
    }

    #[pyo3_asyncio::tokio::test]
    async fn test_multiple_tool_invocations() -> PyResult<()> {
        let data = indoc! {r#"
        # Action
        ```yaml        
        command: SandboxedPython
        input:
            code: |
                print("Hello world 1!")          
        ```
        
        # And another action
        ```yaml        
        command: SandboxedPython
        input:
            code: |
                print("Hello world 2!")          
        ```
        
        # And yet another action
        ```        
        command: SandboxedPython
        input:
            code: |
                print("Hello world 3!")          
        ```
        "#};

        let mut toolbox = Toolbox::default();
        toolbox.add_advanced_tool(PythonTool::default()).await;

        let (tool_name, res) = invoke_tool(toolbox, data).await;
        assert_eq!(tool_name, "SandboxedPython");

        let output = res.unwrap();
        assert_display_snapshot!(output);

        Ok(())
    }

    #[pyo3_asyncio::tokio::test]
    async fn test_python() -> PyResult<()> {
        let mut toolbox = Toolbox::default();
        toolbox.add_tool(DummyTool::default()).await;
        toolbox.add_advanced_tool(PythonTool::default()).await;

        let data = indoc! {r#"```yaml
       command: SandboxedPython
       input:
         code: |           
           args = {
               'blah': "hello"
           }
           output = tools.Dummy(**args)           
          
           something = output['something']                       

           print(f"And the result is: {something}")
       ```
    "#};

        let (tool_name, res) = invoke_tool(toolbox, data).await;
        assert_eq!(tool_name, "SandboxedPython");

        let output = res.unwrap();
        assert_display_snapshot!(output);

        Ok(())
    }
}

#[cfg(feature = "arxiv")]
mod arxiv {
    use indoc::indoc;
    use insta::{assert_display_snapshot, assert_yaml_snapshot};
    use pyo3::PyResult;
    use sapiens::tools::{invoke_tool, Toolbox};
    use sapiens_tools::arxiv::ArxivTool;
    use sapiens_tools::conclude::ConcludeTool;
    use sapiens_tools::python::PythonTool;

    #[pyo3_asyncio::tokio::test]
    async fn test_python_arxiv() -> PyResult<()> {
        let mut toolbox = Toolbox::default();
        toolbox.add_tool(ArxivTool::new().await).await;
        toolbox.add_advanced_tool(PythonTool::default()).await;

        let data = indoc! {r#"```yaml
       command: SandboxedPython
       input:
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

        let (tool_name, res) = invoke_tool(toolbox, data).await;
        assert_eq!(tool_name, "SandboxedPython");

        let output = res.unwrap();
        assert_display_snapshot!(output);

        Ok(())
    }

    #[pyo3_asyncio::tokio::test]
    async fn test_python_arxiv_2() -> PyResult<()> {
        let mut toolbox = Toolbox::default();
        toolbox.add_tool(ArxivTool::new().await).await;
        toolbox.add_advanced_tool(PythonTool::default()).await;

        let data = indoc! {r#"```yaml
       command: SandboxedPython
       input:
         code: |           
               import datetime               
               search_query = f'cat:cs.AI'               
               arxiv_output = tools.Arxiv(search_query=search_query, start=0, max_results=10, sort_by='lastUpdatedDate', sort_order='ascending')               
               papers = arxiv_output['result']               
               conclusion = f"Yes, there are {len(papers)} published papers of AI category on ArXiv this week." if papers else "No, there are no published papers of AI category on ArXiv this week."
               print(conclusion)
       ```
    "#};

        let (tool_name, res) = invoke_tool(toolbox, data).await;
        assert_eq!(tool_name, "SandboxedPython");

        let output = res.unwrap();
        assert_display_snapshot!(output);

        Ok(())
    }

    #[pyo3_asyncio::tokio::test]
    async fn test_python_arxiv_3() -> PyResult<()> {
        let mut toolbox = Toolbox::default();
        toolbox.add_tool(ArxivTool::new().await).await;
        toolbox.add_advanced_tool(PythonTool::default()).await;

        let data = indoc! {r#"```yaml
       command: SandboxedPython
       input:
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

        let (tool_name, res) = invoke_tool(toolbox, data).await;
        assert_eq!(tool_name, "SandboxedPython");

        let output = res.unwrap();
        assert_display_snapshot!(output);

        Ok(())
    }

    #[pyo3_asyncio::tokio::test]
    async fn test_python_arxiv_4() -> PyResult<()> {
        let mut toolbox = Toolbox::default();
        toolbox.add_tool(ArxivTool::new().await).await;
        toolbox.add_terminal_tool(ConcludeTool::default()).await;
        toolbox.add_advanced_tool(PythonTool::default()).await;

        let data = indoc! {r#"```yaml
        command: SandboxedPython
        input:
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

        let (tool_name, res) = invoke_tool(toolbox.clone(), data).await;
        assert_eq!(tool_name, "SandboxedPython");

        let output = res.unwrap();
        assert_display_snapshot!(output);

        let termination_messages = toolbox.termination_messages().await;
        let done = !termination_messages.is_empty();
        assert!(done);

        assert_yaml_snapshot!(termination_messages);

        Ok(())
    }
}
