---
layout: page
title: User Guide
permalink: /user-guide/
---

# BioMedGPS User Guide

This comprehensive guide covers all major features of BioMedGPS, helping you make the most of the platform for your biomedical research needs.

## Table of Contents
- [Dashboard Overview](#dashboard-overview)
- [Knowledge Graph Exploration](#knowledge-graph-exploration)
- [Drug Prediction and Discovery](#drug-prediction-and-discovery)
- [Omics Data Analysis](#omics-data-analysis)
- [Personalized Knowledge Graphs](#personalized-knowledge-graphs)
- [AI-Powered Chat](#ai-powered-chat)
- [Statistics and Analytics](#statistics-and-analytics)
- [User Management](#user-management)

## Dashboard Overview

The dashboard is your starting point in BioMedGPS. It provides:

### Key Metrics
- **Total Entities**: Number of biological entities (genes, diseases, drugs, etc.)
- **Total Relations**: Connections between entities
- **Data Sources**: Integrated databases and publications
- **Recent Activity**: Latest updates and user activities

### Quick Actions
- Search for specific entities
- Access recently viewed content
- Jump to frequently used features
- View system status and health

## Knowledge Graph Exploration

The knowledge graph is the heart of BioMedGPS, allowing you to explore complex biological relationships interactively.

### Basic Search and Navigation

1. **Entity Search**
   - Use the search bar to find genes, diseases, drugs, or other entities
   - Search supports fuzzy matching and synonyms
   - Results show entity type, description, and key statistics

2. **Graph Visualization**
   - Interactive network view of related entities
   - Zoom, pan, and select nodes for detailed information
   - Different colors and shapes represent entity types
   - Edge thickness indicates relationship strength

3. **Node Information Panels**
   - Click any node to view detailed information
   - External links to source databases
   - Related publications and evidence
   - Pathway information and molecular data

### Advanced Features

#### Path Finding
- Find connections between two entities
- Discover intermediate pathways
- Filter by path length and relationship types
- Export pathways for further analysis

#### Similarity Analysis
- Find entities similar to your query
- Based on embeddings and graph structure
- Useful for drug repurposing and target discovery

#### Subgraph Extraction
- Extract focused subgraphs around entities of interest
- Save and share subgraphs with colleagues
- Export to standard formats (GraphML, Cypher)

## Drug Prediction and Discovery

BioMedGPS uses graph neural networks to predict drug-target interactions and identify repurposing opportunities.

### Predict Drugs for Diseases

1. **Select Target Disease**
   - Search for your disease of interest
   - View disease information and current treatments
   - Check available data and evidence level

2. **Configure Prediction Parameters**
   - Set confidence thresholds
   - Choose prediction models
   - Select evaluation metrics

3. **Run Predictions**
   - AI models analyze graph patterns
   - Predictions ranked by confidence
   - Evidence and reasoning provided

4. **Interpret Results**
   - Drug candidates with scores
   - Mechanism of action hypotheses
   - Clinical trial and literature evidence
   - Export results for further validation

### Target Prediction

Similar process for predicting:
- Drug targets for compounds
- Biomarkers for diseases
- Side effects and toxicity
- Drug-drug interactions

## Omics Data Analysis

Integrate and analyze multi-omics data within the knowledge graph context.

### Supported Data Types
- **Genomics**: Gene expression, variants, CNVs
- **Transcriptomics**: RNA-seq, microarray data
- **Proteomics**: Protein expression and modifications
- **Metabolomics**: Metabolite profiles
- **Epigenomics**: DNA methylation, histone modifications

### Analysis Workflows

1. **Data Upload**
   - Support for standard formats (CSV, TSV, Excel)
   - Metadata annotation and validation
   - Quality control and preprocessing options

2. **Statistical Analysis**
   - Differential expression analysis
   - Pathway enrichment analysis
   - Gene set enrichment analysis (GSEA)
   - Network-based analysis

3. **Visualization**
   - Volcano plots and heatmaps
   - Pathway diagrams
   - Network overlays
   - Interactive dashboards

4. **Integration with Knowledge Graph**
   - Map results onto graph structure
   - Identify affected pathways
   - Predict functional consequences
   - Generate hypotheses

### Workflow Management

- **Cromwell Integration**: Run complex bioinformatics workflows
- **Task History**: Track analysis jobs and results
- **Reproducibility**: Save parameters and versions
- **Collaboration**: Share workflows with team members

## Personalized Knowledge Graphs

Create and curate custom knowledge bases tailored to your research focus.

### Knowledge Curation Features

1. **Entity Management**
   - Add custom entities and annotations
   - Import data from external sources
   - Validate and standardize entity names
   - Bulk upload and editing capabilities

2. **Relationship Curation**
   - Define custom relationship types
   - Add evidence and confidence scores
   - Literature-based relationship extraction
   - Quality control and validation

3. **Key Sentence Extraction**
   - Extract important sentences from literature
   - AI-powered relevance scoring
   - Manual curation and validation
   - Integration with knowledge graph

### Collaboration Tools
- **User Permissions**: Control access to curated data
- **Version Control**: Track changes and contributions
- **Export/Import**: Share curated knowledge
- **Statistics**: Monitor curation progress and quality

## AI-Powered Chat

Ask natural language questions and get answers from the integrated knowledge base.

### Supported Query Types
- **Factual Questions**: "What genes are associated with Alzheimer's disease?"
- **Relationship Queries**: "How is BRCA1 related to breast cancer?"
- **Comparison Queries**: "What's the difference between Type 1 and Type 2 diabetes?"
- **Drug Information**: "What are the side effects of metformin?"

### Features
- **Context-Aware**: Remembers conversation history
- **Source Attribution**: Links to supporting evidence
- **Interactive**: Follow-up questions and clarifications
- **Export**: Save conversations and insights

## Statistics and Analytics

Monitor platform usage and knowledge graph metrics.

### Knowledge Graph Statistics
- Entity and relationship counts by type
- Data source contributions
- Growth over time
- Coverage analysis

### Usage Analytics
- User activity patterns
- Popular searches and entities
- Feature usage statistics
- Performance metrics

### Curation Statistics
- Contribution tracking
- Quality metrics
- Progress monitoring
- Collaborative insights

## User Management

### Account Settings
- Profile information and preferences
- Authentication settings
- Privacy controls
- Notification preferences

### Access Control
- Role-based permissions
- Data access levels
- Sharing controls
- Audit trails

### Integration Options
- API access tokens
- Webhook configurations
- Export preferences
- Third-party integrations

## Tips for Effective Use

### Best Practices

1. **Start Broad, Then Focus**
   - Begin with high-level searches
   - Gradually narrow down to specific interests
   - Use filters to manage information overload

2. **Leverage Multiple Views**
   - Switch between graph, table, and detail views
   - Use different visualizations for different insights
   - Export data for external analysis tools

3. **Validate Predictions**
   - Always check evidence and confidence scores
   - Cross-reference with literature
   - Consider experimental validation

4. **Keep Records**
   - Save interesting findings and hypotheses
   - Export results for publications
   - Share insights with collaborators

### Common Workflows

#### Drug Repurposing Research
1. Search for target disease
2. Explore disease mechanisms
3. Predict drug candidates
4. Validate with literature
5. Plan experimental studies

#### Biomarker Discovery
1. Upload omics data
2. Perform differential analysis
3. Map to knowledge graph
4. Identify pathway connections
5. Prioritize candidates

#### Literature Review
1. Use AI chat for initial queries
2. Explore knowledge graph connections
3. Access linked publications
4. Extract key sentences
5. Organize findings

## Getting Help

### Support Resources
- **Documentation**: Comprehensive guides and tutorials
- **FAQ**: Common questions and solutions
- **Community**: GitHub discussions and issues
- **Contact**: Direct support for technical issues

### Training Materials
- **Video Tutorials**: Step-by-step feature walkthroughs
- **Webinars**: Regular training sessions
- **Case Studies**: Real-world usage examples
- **Best Practices**: Tips from experienced users

Remember, BioMedGPS is a powerful platform that grows more valuable as you explore its capabilities. Don't hesitate to experiment with different features and approaches to find what works best for your research needs!