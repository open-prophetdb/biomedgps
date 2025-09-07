---
title: Knowledge Graph Exploration
description: Learn how to navigate and explore the BioMedGPS knowledge graph
category: exploration
order: 1
---

# Knowledge Graph Exploration Guide

The knowledge graph is the core of BioMedGPS, containing millions of biomedical entities and their relationships. This guide will help you effectively explore and navigate this rich resource.

## Getting Started

### Understanding the Graph Structure

The BioMedGPS knowledge graph contains several types of entities:
- **Genes**: Human genes and their variants
- **Diseases**: Medical conditions and disorders
- **Drugs**: Pharmaceutical compounds and treatments
- **Proteins**: Protein sequences and structures
- **Pathways**: Biological pathways and processes
- **Publications**: Scientific literature and evidence

### Basic Navigation

1. **Start with Search**: Use the main search bar to find entities
2. **Explore Connections**: Click on nodes to see their relationships
3. **Use Filters**: Filter by entity type or relationship type
4. **Adjust Layout**: Try different visualization layouts

## Search Strategies

### Simple Search
```
alzheimer disease
BRCA1
aspirin
```

### Advanced Search
- **Exact Match**: Use quotes for exact phrases: "Alzheimer's disease"
- **Entity Type Filter**: Specify entity types: `gene:BRCA1`
- **Wildcard Search**: Use * for partial matches: `alzh*`

### Search Tips
- Use synonyms and alternative names
- Try different spellings (American vs British)
- Include common abbreviations
- Search by database IDs (e.g., DOID:10652)

## Graph Visualization

### Interactive Features
- **Zoom**: Mouse wheel or pinch gestures
- **Pan**: Click and drag to move around
- **Select**: Click nodes to see details
- **Multi-select**: Hold Ctrl/Cmd and click multiple nodes

### Layout Options
- **Force-directed**: Natural clustering of related entities
- **Circular**: Entities arranged in a circle
- **Hierarchical**: Tree-like structure for pathways
- **Grid**: Organized in rows and columns

### Customization
- **Color Coding**: Different colors for entity types
- **Node Size**: Reflects importance or degree
- **Edge Thickness**: Relationship strength or confidence
- **Labels**: Show/hide entity names

## Exploring Relationships

### Relationship Types
- **Gene-Disease**: `gene_associated_with_disease`
- **Drug-Disease**: `drug_treats_disease`
- **Protein-Protein**: `protein_protein_interaction`
- **Drug-Target**: `drug_targets_protein`
- **Pathway**: `gene_participates_in_pathway`

### Finding Connections
1. Select your starting entity
2. Choose relationship types to explore
3. Set maximum distance (1-3 hops recommended)
4. Filter by confidence scores

### Path Discovery
Find paths between any two entities:
1. Select source entity (e.g., a disease)
2. Select target entity (e.g., a drug)
3. Set maximum path length
4. Review discovered pathways

## Practical Examples

### Example 1: Drug Discovery for Alzheimer's Disease

1. **Search for Disease**: "Alzheimer's disease"
2. **Explore Associated Genes**: Look for `gene_associated_with_disease` relationships
3. **Find Drug Targets**: Check which genes are targeted by existing drugs
4. **Identify Candidates**: Look for drugs targeting multiple Alzheimer's genes
5. **Validate Evidence**: Check publication support and clinical trials

### Example 2: Understanding Gene Function

1. **Search for Gene**: "BRCA1"
2. **Check Protein Products**: Follow `gene_codes_for_protein` relationships
3. **Explore Pathways**: Find `participates_in_pathway` connections
4. **Disease Associations**: Review `gene_associated_with_disease` links
5. **Drug Interactions**: Check for targeting drugs

### Example 3: Pathway Analysis

1. **Start with Pathway**: Search for specific pathway (e.g., "p53 signaling")
2. **Identify Components**: Find all genes and proteins in pathway
3. **Disease Context**: Check which diseases affect this pathway
4. **Therapeutic Targets**: Look for druggable targets in pathway
5. **Cross-talk**: Find connections to other pathways

## Advanced Features

### Subgraph Extraction
Create focused views of specific areas:
1. Select entities of interest
2. Set neighborhood distance
3. Apply filters for entity/relationship types
4. Save or export the subgraph

### Similarity Search
Find entities similar to your query:
- Based on graph embeddings
- Structural similarity in the graph
- Functional similarity from annotations
- Literature co-occurrence patterns

### Temporal Analysis
- View how relationships change over time
- Track discovery dates of connections
- Analyze publication trends
- Historical pathway evolution

## Data Quality and Evidence

### Confidence Scores
- **High (0.8-1.0)**: Well-established relationships
- **Medium (0.5-0.8)**: Moderate evidence
- **Low (0.0-0.5)**: Preliminary or weak evidence

### Evidence Sources
- **Literature**: PubMed publications
- **Databases**: Curated biomedical databases
- **Clinical**: Clinical trials and studies
- **Experimental**: Laboratory experiments
- **Computational**: Predicted relationships

### Quality Assessment
- Check number of supporting publications
- Review evidence diversity (multiple sources)
- Consider publication dates (recent vs old)
- Validate with external databases

## Export and Sharing

### Export Options
- **Image**: PNG, SVG for publications
- **Data**: CSV, JSON, GraphML
- **Subgraph**: Cypher queries for Neo4j
- **Report**: PDF summary with visualizations

### Sharing Features
- Generate shareable URLs
- Create public/private collections
- Export citation information
- Integrate with reference managers

## Tips and Best Practices

### Effective Exploration
1. **Start Broad**: Begin with general terms, then narrow down
2. **Use Multiple Entry Points**: Try different starting entities
3. **Follow the Evidence**: Focus on high-confidence relationships
4. **Cross-validate**: Verify findings with multiple sources
5. **Document Insights**: Save interesting findings and hypotheses

### Common Pitfalls
- **Information Overload**: Use filters to focus on relevant data
- **False Connections**: Always check evidence and confidence
- **Outdated Information**: Consider publication dates
- **Bias**: Be aware of research publication biases
- **Over-interpretation**: Correlation doesn't imply causation

### Performance Tips
- **Limit Graph Size**: Use filters for large networks
- **Adjust Visualization**: Simplify complex graphs
- **Use Pagination**: For large result sets
- **Cache Results**: Save frequently used data
- **Optimize Queries**: Be specific in searches

## Integration with Other Features

### Prediction Models
- Use graph exploration to understand predictions
- Verify predicted relationships with graph evidence
- Explore neighborhoods of predicted entities

### Omics Data
- Map analysis results onto the knowledge graph
- Identify affected pathways and processes
- Contextualize findings within known biology

### Literature Chat
- Ask questions about entities you discover
- Get explanations for complex relationships
- Find additional context and background

## Troubleshooting

### Common Issues
- **Slow Loading**: Reduce graph complexity or use filters
- **Missing Entities**: Try alternative names or synonyms
- **Empty Results**: Check spelling and entity types
- **Visualization Problems**: Try different browsers or layouts

### Getting Help
- Use the built-in help tooltips
- Check the FAQ for common questions
- Contact support for technical issues
- Join community discussions for tips

The knowledge graph is a powerful resource that becomes more valuable as you learn to navigate it effectively. Start with simple explorations and gradually work up to more complex analyses as you become familiar with the data and tools.