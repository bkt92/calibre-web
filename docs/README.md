# Calibre-Web Documentation

This directory contains comprehensive documentation for Calibre-Web development, deployment, and usage.

## Documentation Index

### Getting Started

- **[Architecture](architecture.md)** - High-level system architecture and component overview
- **[Development Guide](development.md)** - Setting up development environment and contributing
- **[Deployment Guide](deployment.md)** - Production deployment instructions and best practices

### Core Components

- **[Database](database.md)** - Database schema, models, and access patterns
- **[Authentication](authentication.md)** - Authentication methods and authorization system
- **[Background Tasks](background-tasks.md)** - Asynchronous task processing system
- **[Metadata Providers](metadata-providers.md)** - External metadata source integration
- **[OPDS API](opds-api.md)** - OPDS feed specification for eBook readers

## Quick Reference

### For New Developers

1. Read **[Architecture](architecture.md)** to understand the system
2. Follow **[Development Guide](development.md)** to set up your environment
3. Study **[Database](database.md)** to understand the data model
4. Review **[Authentication](authentication.md)** to understand user management

### For System Administrators

1. Read **[Deployment Guide](deployment.md)** for installation instructions
2. Review **[Architecture](architecture.md)** to understand system components
3. Study **[Authentication](authentication.md)** to configure security
4. Reference **[Background Tasks](background-tasks.md)** to understand task processing

### For API Integration

1. Read **[OPDS API](opds-api.md)** for eBook reader integration
2. Study **[Authentication](authentication.md)** for authentication flows
3. Review **[Architecture](architecture.md)** to understand request flow

## Documentation Conventions

### Code Examples

Code blocks show usage examples:

```python
# Python example
from cps import calibre_db
book = calibre_db.get_book(book_id)
```

```bash
# Shell example
systemctl start calibre-web
```

### Tables

Tables provide structured information:

| Column | Type | Description |
|--------|------|-------------|
| id | Integer | Primary key |

### Diagrams

ASCII diagrams illustrate system flow:

```
Request → Handler → Database → Response
```

### Warnings

⚠️ **Important:** Security warnings are highlighted like this.

## Contributing to Documentation

Documentation improvements are welcome! Please:

1. Keep documentation clear and concise
2. Include examples where helpful
3. Update diagrams when architecture changes
4. Maintain consistent formatting
5. Test all code examples

Submit documentation updates via pull request to the main repository.

## External Resources

- **Main Project:** https://github.com/janeczku/calibre-web
- **Project Wiki:** https://github.com/janeczku/calibre-web/wiki
- **Issue Tracker:** https://github.com/janeczku/calibre-web/issues
- **Discord Community:** https://discord.gg/h2VsJ2NEfB
- **Test Repository:** https://github.com/OzzieIsaacs/calibre-web-test

## Documentation License

Documentation is licensed under the same terms as the Calibre-Web project (GPL v3).
