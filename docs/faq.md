---
layout: page
title: Frequently Asked Questions
permalink: /faq/
---

# Frequently Asked Questions

Find answers to common questions about BioMedGPS. If you can't find what you're looking for, please check our [GitHub Issues](https://github.com/yjcyxky/biomedgps/issues) or create a new issue.

## General Questions

### What is BioMedGPS?

BioMedGPS (Biomedical Graph-based Prediction System) is a comprehensive knowledge graph platform that combines biomedical data integration, graph neural networks, and interactive visualization for drug discovery, disease mechanism analysis, and biomedical knowledge exploration.

### Who can use BioMedGPS?

BioMedGPS is designed for:
- **Biomedical researchers** exploring disease mechanisms and drug interactions
- **Pharmaceutical scientists** working on drug discovery and repurposing
- **Bioinformaticians** analyzing multi-omics data
- **Clinical researchers** investigating biomarkers and therapeutic targets
- **Data scientists** building biomedical AI applications

### Is BioMedGPS free to use?

BioMedGPS is open-source software released under the GNU Affero General Public License v3.0. You can use, modify, and distribute it freely according to the license terms. However, some integrated services (like certain AI models) may require API keys or subscriptions.

### What data sources does BioMedGPS integrate?

BioMedGPS integrates data from multiple sources including:
- DRKG (Drug Repurposing Knowledge Graph)
- PubMed literature
- UniProt protein database
- KEGG pathways
- Gene Ontology
- ChEMBL bioactivity data
- STRING protein interactions
- DisGeNET disease-gene associations

## Technical Questions

### What are the system requirements for running BioMedGPS?

**Minimum Requirements:**
- 8GB RAM
- 4 CPU cores
- 50GB available disk space
- PostgreSQL 12+
- Neo4j 4.0+

**Recommended:**
- 16GB+ RAM
- 8+ CPU cores
- SSD storage
- Docker for easy deployment

### Can I run BioMedGPS on cloud platforms?

Yes, BioMedGPS can be deployed on various cloud platforms:
- **AWS**: EC2 instances with RDS and managed Neo4j
- **Google Cloud**: Compute Engine with Cloud SQL
- **Azure**: Virtual machines with managed databases
- **Docker**: Containerized deployment on any platform

### How do I update BioMedGPS to the latest version?

For source installations:
```bash
git pull origin main
make build-mac  # or build-linux
```

For Docker deployments:
```bash
docker-compose pull
docker-compose up -d
```

Always backup your data before updating and check the changelog for breaking changes.

## Data and Import Questions

### How do I import my own data into BioMedGPS?

You can import custom data through several methods:

1. **Web Interface**: Use the Knowledge Curation pages to add entities and relationships manually
2. **Bulk Import**: Use the CLI tool for large datasets:
   ```bash
   biomedgps-cli importdb -f your_data.tsv -t entity -D
   ```
3. **API**: Programmatically import data using the REST API
4. **Direct Database**: Insert data directly into PostgreSQL tables

### What file formats are supported for data import?

- **Tabular Data**: CSV, TSV, Excel (.xlsx, .xls)
- **Bioinformatics**: VCF, GFF, BED formats
- **Structured Data**: JSON, XML
- **Database**: Direct PostgreSQL connections

### How do I handle large datasets?

For large datasets (>1M records):
1. Use batch processing with the CLI tool
2. Split large files into smaller chunks
3. Use database indexes for better performance
4. Consider using background processing for imports
5. Monitor system resources during import

### Can I integrate proprietary or confidential data?

Yes, you can integrate proprietary data while maintaining confidentiality:
- Use local deployment to keep data on-premises
- Implement custom access controls
- Use data anonymization features
- Set up private user groups and permissions
- Consider encryption for sensitive data

## Usage and Features

### How do I search for entities in the knowledge graph?

1. **Simple Search**: Use the main search bar on any page
2. **Advanced Search**: Use filters for entity type, data source, etc.
3. **API Search**: Use the `/entities` endpoint programmatically
4. **Graph Navigation**: Start from one entity and explore neighbors
5. **Similarity Search**: Find entities similar to your query using embeddings

### What types of predictions can BioMedGPS make?

BioMedGPS can predict:
- **Drug Repurposing**: Existing drugs for new indications
- **Target Discovery**: Therapeutic targets for diseases
- **Biomarkers**: Diagnostic and prognostic markers
- **Drug-Drug Interactions**: Potential adverse interactions
- **Side Effects**: Adverse reactions for drug combinations
- **Pathway Analysis**: Affected biological pathways

### How accurate are the predictions?

Prediction accuracy varies by model and data type:
- **Drug Repurposing**: ~70-85% precision for top predictions
- **Target Discovery**: ~60-75% precision depending on disease
- **Biomarkers**: Validation required for clinical use
- **Side Effects**: Used for hypothesis generation

Always validate predictions experimentally and check confidence scores.

### Can I customize the prediction models?

Yes, several customization options are available:
- Adjust prediction thresholds
- Select specific model versions
- Train models on custom datasets
- Use ensemble methods for better accuracy
- Implement custom scoring functions

### How do I visualize complex networks?

BioMedGPS provides multiple visualization options:
- **Interactive Graphs**: Force-directed layouts with zooming/panning
- **Hierarchical Views**: Tree-like structures for pathways
- **Circular Layouts**: For symmetrical network views
- **Multi-layer**: Separate different relationship types
- **Custom Filtering**: Show/hide specific node or edge types

## Troubleshooting

### Common Installation Issues

**Database Connection Errors**
```
Error: Connection refused (PostgreSQL/Neo4j)
```
**Solution:**
1. Verify databases are running: `docker ps`
2. Check connection strings in environment variables
3. Ensure firewall allows connections
4. Verify credentials and database names

**Build Failures**
```
Error: cargo build failed
```
**Solution:**
1. Update Rust: `rustup update`
2. Clear build cache: `cargo clean`
3. Check system dependencies
4. Review error logs for specific issues

**Frontend Issues**
```
Error: Module not found
```
**Solution:**
1. Verify Node.js version (must be 16.13.1)
2. Clear cache: `yarn cache clean`
3. Reinstall dependencies: `rm -rf node_modules && yarn install`
4. Check for conflicting global packages

### Performance Issues

**Slow Query Performance**
- Add database indexes for frequently queried fields
- Use query pagination for large result sets
- Optimize graph traversal depth
- Consider caching for repeated queries
- Monitor database performance metrics

**High Memory Usage**
- Increase available RAM
- Optimize graph loading strategies
- Use streaming for large datasets
- Implement garbage collection tuning
- Monitor memory usage patterns

**Slow Predictions**
- Use GPU acceleration if available
- Reduce batch sizes for memory constraints
- Cache model embeddings
- Use faster model variants
- Consider distributed computing

### Common Usage Issues

**Authentication Problems**
- Check Auth0 configuration
- Verify client ID and domain settings
- Clear browser cookies and cache
- Check network connectivity
- Review CORS settings

**Data Import Failures**
- Validate file formats and encoding
- Check column headers and data types
- Verify entity IDs and relationships
- Monitor import logs for errors
- Use smaller batch sizes

**Visualization Problems**
- Update browser to latest version
- Clear browser cache
- Disable browser extensions
- Check WebGL support
- Try different layout algorithms

## Advanced Topics

### How do I set up high availability?

For production deployments:
1. Use database clustering (PostgreSQL HA, Neo4j Causal Cluster)
2. Implement load balancing with multiple application instances
3. Set up automated backups and disaster recovery
4. Monitor system health and performance
5. Use container orchestration (Kubernetes, Docker Swarm)

### Can I extend BioMedGPS with custom plugins?

Yes, BioMedGPS supports extensions through:
- **API Integration**: Build applications using the REST API
- **Custom Models**: Implement new prediction algorithms
- **Data Connectors**: Add support for new data sources
- **Visualization Plugins**: Create custom graph layouts
- **Workflow Integration**: Connect with external tools

### How do I contribute to BioMedGPS development?

We welcome contributions! Here's how to get started:
1. Fork the repository on GitHub
2. Set up development environment
3. Review open issues and feature requests
4. Submit pull requests with improvements
5. Help with documentation and testing

### How do I cite BioMedGPS in publications?

When using BioMedGPS in research, please cite:
```
BioMedGPS: A knowledge graph platform for biomedical discovery
GitHub repository: https://github.com/yjcyxky/biomedgps
```

Include the version number and any specific features you used.

## Getting More Help

### Documentation Resources
- **User Guide**: Comprehensive feature documentation
- **API Reference**: Technical documentation for developers
- **Video Tutorials**: Step-by-step walkthroughs
- **GitHub Wiki**: Additional technical notes

### Community Support
- **GitHub Discussions**: General questions and ideas
- **GitHub Issues**: Bug reports and feature requests
- **Stack Overflow**: Tag questions with `biomedgps`
- **Twitter**: Follow @biomedgps for updates

### Professional Support
For enterprise deployments or custom development:
- Commercial support options available
- Consulting services for large-scale deployments
- Custom feature development
- Training and workshops

### Reporting Bugs

When reporting bugs, please include:
1. BioMedGPS version number
2. Operating system and browser
3. Steps to reproduce the issue
4. Error messages and logs
5. Screenshots if applicable

Use the [GitHub issue template](https://github.com/yjcyxky/biomedgps/issues/new) for bug reports.

---

**Still have questions?** Don't hesitate to reach out through our GitHub repository or community channels. The BioMedGPS team and community are here to help!