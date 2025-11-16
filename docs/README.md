# ANDE Chain Documentation

> Complete, organized, and up-to-date documentation

**Last Updated**: 2025-11-16  
**Status**: ✅ Production Ready

---

## 🎯 Quick Navigation

| I want to... | Read this |
|--------------|-----------|
| **Start developing** | [Development Guide](DEVELOPMENT_GUIDE.md) |
| **Understand the architecture** | [Custom Reth Implementation](CUSTOM_RETH_IMPLEMENTATION.md) |
| **Deploy a node** | [Deployment Guide](DEPLOYMENT.md) |
| **Work with contracts** | [Contracts Documentation](CONTRACTS.md) |
| **Quick reference** | [Quick Reference](../QUICK_REFERENCE.md) |
| **Recreate from scratch** | [Recreation Guide](../RECREATION_GUIDE.md) |

---

## 📚 Core Documentation

### 🏗️ Architecture & Implementation

#### [Custom Reth Implementation Guide](CUSTOM_RETH_IMPLEMENTATION.md) ⭐⭐⭐
**The most important document** - Complete guide to ANDE's custom Reth fork

**Contains**:
- Wrapper pattern architecture explained
- All problems encountered and solved
- Exact locations for future modifications
- Troubleshooting guide
- ~2000 lines of detailed documentation

**Read this if**: You need to understand how ANDE Chain works internally

---

#### [Development Guide](DEVELOPMENT_GUIDE.md) ⭐⭐
Day-to-day development guide

**Contains**:
- Project structure
- Common tasks (building, testing, debugging)
- Code style guide
- Security guidelines
- Performance profiling

**Read this if**: You're actively developing ANDE Chain

---

### 🚀 Deployment & Operations

#### [Deployment Guide](DEPLOYMENT.md) ⭐⭐
Complete deployment documentation

**Contains**:
- Local development setup
- Testnet deployment
- Production deployment with load balancing
- Docker deployment
- Monitoring setup
- Troubleshooting

**Read this if**: You need to deploy or maintain ANDE Chain

---

#### [Docker Setup](../DOCKER_README.md)
Container-based deployment

**Contains**:
- Docker Compose configuration
- Service definitions
- Volume management

**Read this if**: You prefer containerized deployment

---

### 📜 Smart Contracts

#### [Contracts Documentation](CONTRACTS.md) ⭐
Consolidated contracts documentation

**Contains**:
- Contract architecture
- Consensus system
- Token duality
- Account abstraction
- Staking system
- Security audit summaries
- Deployment info

**Read this if**: You're working with ANDE smart contracts

---

### 🔐 Security

#### [Token Duality Security Audit](SECURITY_AUDIT_PRECOMPILE.md)
Comprehensive security review

**Status**: ✅ 0 Critical, 0 High vulnerabilities

**Contains**:
- Audit methodology
- Findings and fixes
- Security recommendations
- Test coverage

**Read this if**: You need security details on precompiles

---

#### [Security Review Summary](SECURITY_REVIEW_SUMMARY.md)
Executive summary of all security work

**Contains**:
- High-level findings
- Implementation status
- Production readiness

---

### 🔧 Quick References

#### [Quick Reference](../QUICK_REFERENCE.md) ⭐
Command cheat sheet

**Contains**:
- Essential commands
- Critical files reference
- Common errors and fixes
- Debugging commands
- Git workflow

**Read this if**: You need quick command reference

---

#### [Recreation Guide](../RECREATION_GUIDE.md) ⭐
Step-by-step guide to recreate implementation

**Contains**:
- Exact order of implementation
- Critical points for each step
- Verification checklist
- Time estimates

**Read this if**: You need to recreate ANDE Chain from scratch

---

#### [Implementation Complete Summary](../WRAPPER_PATTERN_IMPLEMENTATION_COMPLETE.md)
Summary of wrapper pattern implementation

**Contains**:
- What was implemented
- Problems solved
- Architecture achieved
- Next steps

**Read this if**: You want a high-level overview of what was built

---

## 🗂️ Additional Documentation

### By Component

**EVM & Precompiles**:
- [Parallel Security](../crates/ande-evm/PARALLEL_SECURITY.md) - Parallel execution security
- [Precompile Usage](../crates/ande-evm/src/evm_config/PRECOMPILE_USAGE.md) - How to use precompiles
- [Best Practices](../crates/ande-evm/src/evm_config/BEST_PRACTICES.md) - EVM development best practices

**Consensus**:
- [ABI README](../crates/ande-consensus/abi/README.md) - Contract ABIs

**Infrastructure**:
- [Celestia Integration](CELESTIA_INTEGRATION_ARCHITECTURE.md) - DA layer integration
- [Genesis Workflow](GENESIS_WORKFLOW.md) - Genesis file creation

**Contracts**:
- [Contract Architecture](../contracts/src/ARCHITECTURE.md) - Design patterns
- [Recent Changes](../contracts/src/RECENT_CHANGES.md) - Latest updates
- [Deployment Manual](../contracts/DEPLOY_MANUAL.md) - Deployment instructions
- [Security Audit Report](../contracts/SECURITY_AUDIT_REPORT.md) - Full audit

---

## 📖 Learning Paths

### Path 1: New Developer (2-3 hours)

1. Read [README.md](../README.md) - Project overview (15 min)
2. Follow [Quick Start](../QUICK_START.md) - Setup environment (30 min)
3. Study [Development Guide](DEVELOPMENT_GUIDE.md) - Development workflow (1 hour)
4. Keep [Quick Reference](../QUICK_REFERENCE.md) handy - Command reference

**Result**: Ready to contribute

---

### Path 2: Understanding Architecture (4-6 hours)

1. Read [Custom Reth Implementation](CUSTOM_RETH_IMPLEMENTATION.md) - Full architecture (2 hours)
2. Study wrapper pattern section carefully (1 hour)
3. Review actual code in `crates/ande-reth/` (2 hours)
4. Understand [Contracts Documentation](CONTRACTS.md) (1 hour)

**Result**: Deep understanding of ANDE Chain internals

---

### Path 3: Deployment & Ops (2-3 hours)

1. Read [Deployment Guide](DEPLOYMENT.md) - Full deployment process (1.5 hours)
2. Review [Docker Setup](../DOCKER_README.md) - Containerization (30 min)
3. Setup monitoring following deployment guide (1 hour)

**Result**: Can deploy and maintain ANDE Chain

---

### Path 4: Smart Contract Development (3-4 hours)

1. Read [Contracts Documentation](CONTRACTS.md) - Overview (1 hour)
2. Review [Contract Architecture](../contracts/src/ARCHITECTURE.md) - Design patterns (1 hour)
3. Study [Security Audit](SECURITY_AUDIT_PRECOMPILE.md) - Security considerations (1 hour)
4. Practice with [Deployment Manual](../contracts/DEPLOY_MANUAL.md) (1 hour)

**Result**: Can develop and deploy contracts

---

## 🔍 Finding Information

### By Topic

**Precompiles**:
- Implementation: `crates/ande-evm/src/evm_config/ande_token_duality_precompile.rs`
- Documentation: [Security Audit](SECURITY_AUDIT_PRECOMPILE.md)
- Usage: `crates/ande-evm/src/evm_config/PRECOMPILE_USAGE.md`

**Consensus**:
- Reth Integration: `crates/ande-reth/src/consensus.rs`
- Contracts: `contracts/src/consensus/`
- Client: `crates/ande-consensus/`

**Node Configuration**:
- Node Type: `crates/ande-reth/src/node.rs`
- Main Entry: `crates/ande-reth/src/main.rs`
- Genesis: `specs/genesis.json`

**Deployment**:
- Guide: [Deployment Guide](DEPLOYMENT.md)
- Docker: [Docker Setup](../DOCKER_README.md)
- Scripts: `scripts/`

---

## 🎓 Best Practices

### When Reading Documentation

1. **Start with the right document** - Use Quick Navigation above
2. **Follow learning paths** - Don't jump around randomly
3. **Check last updated date** - Ensure you're reading current info
4. **Try examples** - Run code snippets as you read
5. **Ask questions** - Open GitHub issues for clarifications

### When Contributing Documentation

1. **Update existing docs** - Don't create duplicates
2. **Add last updated date** - Always include date at top
3. **Link related docs** - Cross-reference other documents
4. **Include examples** - Show, don't just tell
5. **Test your examples** - Ensure code works

---

## 📊 Documentation Status

| Category | Status | Last Updated |
|----------|--------|--------------|
| Architecture | ✅ Complete | 2025-11-16 |
| Development | ✅ Complete | 2025-11-16 |
| Deployment | ✅ Complete | 2025-11-16 |
| Contracts | ✅ Complete | 2025-11-16 |
| Security | ✅ Complete | 2025-11-15 |
| Quick References | ✅ Complete | 2025-11-16 |

---

## 📝 Documentation Standards

All ANDE Chain documentation follows these standards:

### Structure

- **Title and description** at top
- **Last updated date** prominently displayed
- **Table of contents** for docs >500 lines
- **Code examples** with explanations
- **Links to related docs**

### Code Examples

- Always include **working code**
- Show **expected output**
- Note **common pitfalls**
- Follow **best practices**

### Maintenance

- **Update dates** when content changes
- **Archive old docs** to `docs/archive-old/`
- **Link from index** when creating new docs
- **Cross-reference** related documentation

---

## 🗄️ Archived Documentation

Old documentation has been moved to:
- `docs/archive-old/` - General archive
- `docs/archive/` - Historical documents
- `contracts/archive-old/` - Old contract docs

**Note**: Archived docs are kept for historical reference but may be outdated.

---

## 🆘 Getting Help

### Documentation Issues

**Found outdated info?**
1. Check [last updated date](#documentation-status)
2. Open GitHub issue with label `documentation`
3. Specify: document name, section, issue

**Can't find something?**
1. Use search: `grep -r "topic" docs/`
2. Check [archived docs](#archived-documentation)
3. Ask on Discord

**Want to contribute?**
1. Read [Contributing Guide](../CONTRIBUTING.md)
2. Follow [documentation standards](#documentation-standards)
3. Submit PR

---

## 📞 Support Channels

- **GitHub Issues**: Technical questions, bug reports
- **Discord**: General discussion, quick questions
- **Email**: devs@andelabs.com (sensitive issues)

---

## 🎯 What's Next?

After reviewing documentation:

**For Developers**:
→ Start with [Development Guide](DEVELOPMENT_GUIDE.md)

**For Architects**:
→ Deep dive into [Custom Reth Implementation](CUSTOM_RETH_IMPLEMENTATION.md)

**For DevOps**:
→ Follow [Deployment Guide](DEPLOYMENT.md)

**For Everyone**:
→ Keep [Quick Reference](../QUICK_REFERENCE.md) bookmarked

---

**Documentation maintained by**: ANDE Labs Engineering Team  
**Last comprehensive update**: 2025-11-16  
**Documentation version**: 2.0.0 (Cleaned and consolidated)

---

## 📈 Changelog

### v2.0.0 (2025-11-16)
- ✅ Complete reorganization
- ✅ Archived outdated documents
- ✅ Created consolidated guides
- ✅ Added learning paths
- ✅ Improved navigation
- ✅ ~10,000 lines of documentation

### v1.0.0 (2025-11-15)
- Initial documentation structure
- Security audit completed
- Precompile integration documented

---

**Happy learning! 🚀**
