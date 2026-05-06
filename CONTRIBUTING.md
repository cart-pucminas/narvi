# Contributing to Narvi

It is good to hear that you are willing to contribute with the Narvi project! This guide covers the necessary information to help you to become a contributor.

## Ways of Contributing

- **Bug reports:** identifying and documenting reproducible bugs;
- **Feature suggestion:** proposing new functionalities or enhancements;
- **Code submission:** submitting Pull Requests for features, fixes, or refactors;
- **Documentation:** improving README, wikis or inline code comments;
- **Feedback & questions:** Engaging in discussions about the projects direction and reviewing Pull Requests.

## Issue Submission

An issue must be submitted for:

- Reporting bugs;
- Discussing new features or refactors;
- Asking technical questions.

Every issue must include at least one of the following primary labels:

- `bug`;
- `enhancement`;
- `question`.

Secondary labels may be added for further context.

## Commit Rules

To maintain a clean and readable history, commits should be atomic (small, logical changes). Squashing multiple minor commits before merging is encouraged.

Commit message format:

```
<type>(<scope>)<!>: <description>
```

The following types are available:

- `feat`: addition of a new feature (requires scope);
- `fix`: correction of bugs (requires scope). Moreover, a fix must reference an open issue labeled as `bug`;
- `build`: changes related to the building process (does not require scope);
- `refactor`: any kind of structural or semantic change that does not add nor modifies existent behavior (requires scope);
- `test`: any kind of testing (requires scope);
- `docs`: any kind of change related to documentation (does not require scope).

The scope must consist of a short text that describes the codebase section affected by the commit. It should be preferably a module name.

If the commit consists of a breaking change, that is, something that adds layers of incompatibility to previous versions, the closing parenthesis succeeding the scopt must follow an exclamation point (`!`).

The description is a short summary of the code changes, and it must not exceed 75 characters in total, disregarding the preceding space.

Examples:
```
feat(hart)!: added support for RV64A
fix(memory): fixed TLB bug #10
docs: added contributing guidelines
```
