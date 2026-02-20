# Create PRP

## Feature file: $ARGUMENTS

Generate a complete PRP for general feature implementation with thorough research. Ensure context is passed to the AI agent to enable self-validation and iterative refinement. Read the feature file first to understand what needs to be created, how the examples provided help, and any other considerations.

The AI agent only gets the context you are appending to the PRP and training data. Assume the AI agent has access to the codebase and the same knowledge cutoff as you, so its important that your research findings are included or referenced in the PRP. The Agent has Websearch capabilities, so pass urls to documentation and examples.

## MANDATORY FIRST STEP

**ALWAYS** read and reference `PROJECT_OVERVIEW.md` to understand:
- Project architecture and technology stack
- State management patterns (TanStack Query + Zustand)
- API integration patterns (Next.js API routes)
- Theming and component conventions
- Authentication flow
- Existing implementation phases and patterns

## Research Process

### Research Management
- **ALWAYS** check `research/` folder first before doing external research
- Save all research findings in structured markdown files in `research/` folder
- Reference existing research files when available to avoid duplication
- Update research files with new findings when relevant

### Research Folder Structure
```
research/
├── architecture/           # Project architecture patterns
├── libraries/             # Library-specific documentation and patterns
│   ├── nextjs/
│   ├── tanstack-query/
│   ├── zustand/
│   └── shadcn-ui/
├── patterns/              # Common implementation patterns
├── features/              # Feature-specific research
├── authentication/        # Auth patterns and web3 integration
└── ui-components/         # Component patterns and theming
```

1. **Check Existing Research** (MANDATORY FIRST STEP)
   - Search `research/` folder for relevant existing findings
   - Check `research/features/` for similar feature implementations
   - Review `research/patterns/` for applicable patterns
   - Look in `research/libraries/` for relevant library documentation
   - Note gaps in existing research for focused new research

2. **Project Architecture Review** (MANDATORY)
   - Read PROJECT_OVERVIEW.md for architectural context
   - Save architectural insights to `research/architecture/current-state.md`
   - Identify how feature fits into existing system
   - Respect established data flow patterns
   - Check authentication requirements and user context needs
   - Verify theming and white-label customization requirements

3. **Codebase Analysis**
   - Search for similar features/patterns in the codebase
   - Save patterns to `research/patterns/{pattern-name}.md`
   - Identify existing API routes and services to follow as patterns
   - Review existing stores for state management patterns
   - Check hooks for data fetching patterns
   - Note existing conventions (TypeScript patterns, component library usage)
   - Check test patterns for validation approach

4. **Documentation Research**
   - Check existing files in `research/libraries/` before external search
   - Check CLAUDE.md for project-specific instructions
   - Look for relevant documentation in docs/ directory if it exists
   - Update `research/libraries/{library}/` with new findings
   - Include specific URLs to documentation sections

5. **External Research** (Only after checking existing research)
   - Search for similar features/patterns online ONLY if not in research folder
   - Save findings to appropriate `research/` subfolder
   - Library documentation (include specific URLs)
   - Implementation examples (GitHub/StackOverflow/blogs)
   - Best practices and common pitfalls
   - Web3 and authentication patterns if applicable
   - **Format**: Save as `research/{category}/{topic}.md` with:
     ```markdown
     # {Topic} Research

     ## Summary
     Brief overview of findings

     ## Key Points
     - Important findings
     - Relevant patterns
     - Best practices

     ## Documentation Links
     - [Link 1](url) - Description
     - [Link 2](url) - Description

     ## Code Examples
     ```{language}
     // Relevant code examples
     ```

     ## Implementation Notes
     - Specific to our project
     - Integration considerations

     ## Last Updated
     {Date}
     ```

6. **Research Consolidation**
   - Create feature-specific research file: `research/features/{feature-name}.md`
   - Consolidate all relevant findings from different research files
   - Include cross-references to other research files
   - Highlight gaps that need user clarification

7. **User Clarification** (if needed)
   - Specific patterns to mirror and where to find them?
   - Integration requirements and where to find them?
   - API endpoints and data flow requirements?

## PRP Generation

Using PRPs/templates/prp_base.md as template:

### Critical Context to Include
- **Research Findings**: Reference relevant research files from `research/` folder with direct file paths
- **Project Overview**: Reference to PROJECT_OVERVIEW.md for architectural context
- **Data Flow Patterns**: API routes → TanStack Query → Zustand store integration
- **State Management**: Existing stores and how to extend them
- **API Integration**: API route patterns and backend integration
- **Authentication Context**: Wallet connection patterns
- **Documentation**: URLs with specific sections from research files
- **Code Examples**: Real snippets from existing codebase patterns and research files
- **Theming Requirements**: White-label customization considerations
- **Component Patterns**: Component library integration examples from research
- **Gotchas**: Library quirks, version issues, web3 considerations from research files
- **Feature Research**: Link to consolidated feature research file `research/features/{feature-name}.md`

### Implementation Blueprint
- Start with architectural overview showing data flow
- Reference real files for established patterns (stores, hooks, API routes, components)
- Follow existing state management integration patterns
- Respect authentication and user context requirements
- Include error handling strategy following existing patterns
- Consider white-label theming requirements
- List tasks to be completed to fulfill the PRP in the order they should be completed
- Ensure TypeScript types are properly defined and exported

### Validation Gates (Must be Executable)
```bash
# TypeScript type checking
npx tsc --noEmit

# ESLint checking
npm run lint

# Build verification
npm run build

# Development server start (optional verification)
npm run dev
```

### Required Architecture Compliance
- [ ] Follows established data flow patterns
- [ ] Uses existing authentication patterns
- [ ] Implements proper TypeScript types with exports
- [ ] Integrates with existing stores or creates new ones following patterns
- [ ] Uses component library with proper theming support
- [ ] Follows white-label customization requirements
- [ ] Includes proper error handling and loading states
- [ ] Respects existing API integration patterns

---

**CRITICAL**: After researching and exploring the codebase, **think hard** about the PRP and plan your approach before writing.

## Output
Save as: `PRPs/{feature-name}.md`

## Quality Checklist
- [ ] Research folder checked first — existing research reviewed before external searches
- [ ] Research findings saved — all new findings saved to structured `research/` files
- [ ] Feature research consolidated — `research/features/{feature-name}.md` created
- [ ] PROJECT_OVERVIEW.md referenced and architectural context provided
- [ ] Data flow patterns clearly documented
- [ ] Existing stores and state management patterns identified from research
- [ ] Authentication and user context requirements addressed
- [ ] Theming and white-label customization requirements considered
- [ ] Component patterns referenced from research files
- [ ] API integration patterns documented
- [ ] TypeScript types and patterns established
- [ ] Validation gates are executable by AI
- [ ] References existing codebase patterns with specific file examples
- [ ] Research files referenced in PRP with direct file paths
- [ ] Clear implementation path following established architecture
- [ ] Error handling and loading states documented
- [ ] All necessary external documentation URLs included and saved to research files

Score the PRP on a scale of 1-10 (confidence level to succeed in one-pass implementation).

**Goal**: One-pass implementation success through comprehensive context that respects the established architecture, state management patterns, and integration requirements.
