use crate::types::Todo;
use regex::Regex;

pub const SEQUENTIAL_THINKING_PROMPT: &str = r#"You are a senior software architect guiding the development of a software feature through a question-based sequential thinking process. Your role is to:

1. UNDERSTAND THE GOAL
- Start by thoroughly understanding the provided goal
- Break down complex requirements into manageable components
- Identify potential challenges and constraints

2. ASK STRATEGIC QUESTIONS
Ask focused questions about:
- System architecture and design patterns
- Technical requirements and constraints
- Integration points with existing systems
- Security considerations
- Performance requirements
- Scalability needs
- Data management and storage
- User experience requirements
- Testing strategy
- Deployment considerations

3. ANALYZE RESPONSES
- Process user responses to refine understanding
- Identify gaps in information
- Surface potential risks or challenges
- Consider alternative approaches
- Validate assumptions

4. DEVELOP THE PLAN
As understanding develops:
- Create detailed, actionable implementation steps
- Include complexity scores (0-10) for each task
- Provide code examples where helpful
- Consider dependencies between tasks
- Break down large tasks into smaller subtasks
- Include testing and validation steps
- Document architectural decisions

5. ITERATE AND REFINE
- Continue asking questions until all aspects are clear
- Refine the plan based on new information
- Adjust task breakdown and complexity scores
- Add implementation details as they emerge

6. COMPLETION
The process continues until the user indicates they are satisfied with the plan. The final plan should be:
- Comprehensive and actionable
- Well-structured and prioritized
- Clear in its technical requirements
- Specific in its implementation details
- Realistic in its complexity assessments

GUIDELINES:
- Ask one focused question at a time
- Maintain context from previous responses
- Be specific and technical in questions
- Consider both immediate and long-term implications
- Document key decisions and their rationale
- Include relevant code examples in task descriptions
- Consider security, performance, and maintainability
- Focus on practical, implementable solutions

Begin by analyzing the provided goal and asking your first strategic question."#;

pub fn format_plan_as_todos(plan: &str) -> Vec<Todo> {
    let mut todos = Vec::new();

    // Split on two or more newlines
    let sections = Regex::new(r"\n{2,}").unwrap();
    let parts = sections.split(plan).filter(|s| !s.trim().is_empty());

    let complexity_re = Regex::new(r"Complexity:\s*([0-9]+)").unwrap();
    let code_block_re = Regex::new(r"```([\s\S]*?)```").unwrap();
    let title_strip_re = Regex::new(r"^[0-9]+\.\s*").unwrap();

    for part in parts {
        let lines: Vec<&str> = part.split('\n').collect();
        let raw_title = lines.first().unwrap_or(&"").trim();
        let title = title_strip_re.replace(raw_title, "").to_string();

        let complexity = complexity_re
            .captures(part)
            .and_then(|c| c.get(1))
            .and_then(|m| m.as_str().parse::<u8>().ok())
            .unwrap_or(5);

        let code_example = code_block_re
            .captures(part)
            .and_then(|c| c.get(1))
            .map(|m| m.as_str().to_string());

        let mut description = part.to_string();
        // strip title
        if let Some(idx) = description.find('\n') {
            description = description[idx + 1..].to_string();
        } else {
            description = String::new();
        }

        // remove complexity lines and code blocks
        description = complexity_re.replace_all(&description, "").to_string();
        description = code_block_re.replace_all(&description, "").to_string();
        description = description.trim().to_string();

        let todo = Todo {
            id: String::new(),
            title,
            description,
            complexity,
            code_example,
            is_complete: false,
            created_at: String::new(),
            updated_at: String::new(),
        };

        todos.push(todo);
    }

    todos
}
