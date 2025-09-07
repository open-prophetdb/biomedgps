---
layout: page
title: API Reference
permalink: /api-reference/
---

# API Reference

BioMedGPS provides a comprehensive RESTful API that allows developers to programmatically access all platform functionality. The API is built using OpenAPI 3.0 specification and provides interactive documentation.

## Getting Started with the API

### Base URL
```
https://your-biomedgps-instance.com/api
```

For local development:
```
http://localhost:8888
```

### Authentication
BioMedGPS uses JWT-based authentication. Include your token in the Authorization header:

```bash
curl -H "Authorization: Bearer YOUR_JWT_TOKEN" \
     https://your-instance.com/api/entities
```

### OpenAPI Documentation
Interactive API documentation is available at:
```
https://your-instance.com/docs
```

This provides:
- Complete endpoint documentation
- Request/response schemas
- Interactive API testing
- Code generation for multiple languages

## Core API Endpoints

### Entities API

#### Search Entities
Find biological entities (genes, diseases, drugs, etc.) in the knowledge graph.

```http
GET /entities?query={search_term}&entity_type={type}&limit={n}
```

**Parameters:**
- `query` (string): Search term or entity name
- `entity_type` (optional): Filter by entity type (gene, disease, drug, etc.)
- `limit` (optional): Maximum number of results (default: 20)

**Example:**
```bash
curl "http://localhost:8888/entities?query=alzheimer&entity_type=disease&limit=10"
```

**Response:**
```json
{
  "entities": [
    {
      "id": "DOID:10652",
      "name": "Alzheimer's disease",
      "entity_type": "disease",
      "description": "A tauopathy that is characterized by...",
      "synonyms": ["AD", "Alzheimer disease"],
      "attributes": {
        "mesh_id": "D000544",
        "umls_cui": "C0002395"
      }
    }
  ],
  "total": 1,
  "page": 1,
  "limit": 10
}
```

#### Get Entity Details
Retrieve detailed information about a specific entity.

```http
GET /entities/{entity_id}
```

**Example:**
```bash
curl "http://localhost:8888/entities/DOID:10652"
```

### Relationships API

#### Get Entity Relationships
Find relationships for a specific entity.

```http
GET /entities/{entity_id}/relationships?relation_type={type}&limit={n}
```

**Parameters:**
- `relation_type` (optional): Filter by relationship type
- `limit` (optional): Maximum number of relationships

**Example:**
```bash
curl "http://localhost:8888/entities/DOID:10652/relationships?relation_type=drug_treats_disease"
```

#### Find Paths Between Entities
Discover paths connecting two entities in the knowledge graph.

```http
GET /paths?source={source_id}&target={target_id}&max_length={n}
```

**Example:**
```bash
curl "http://localhost:8888/paths?source=DOID:10652&target=DB00843&max_length=3"
```

### Predictions API

#### Drug Prediction
Predict drugs for a given disease or target.

```http
POST /predictions/drugs
```

**Request Body:**
```json
{
  "entity_id": "DOID:10652",
  "entity_type": "disease",
  "model": "graph_neural_network",
  "threshold": 0.7,
  "limit": 50
}
```

**Response:**
```json
{
  "predictions": [
    {
      "drug_id": "DB00843",
      "drug_name": "Donepezil",
      "score": 0.85,
      "confidence": "high",
      "evidence": [
        {
          "type": "clinical_trial",
          "source": "NCT00000173",
          "description": "Phase 3 trial for Alzheimer's disease"
        }
      ]
    }
  ],
  "metadata": {
    "model_version": "v2.1.0",
    "timestamp": "2024-01-15T10:30:00Z"
  }
}
```

#### Target Prediction
Predict therapeutic targets for diseases.

```http
POST /predictions/targets
```

#### Biomarker Prediction
Identify potential biomarkers for diseases or conditions.

```http
POST /predictions/biomarkers
```

### Knowledge Graph API

#### Subgraph Extraction
Extract a subgraph around entities of interest.

```http
POST /subgraph
```

**Request Body:**
```json
{
  "entities": ["DOID:10652", "HGNC:620"],
  "max_distance": 2,
  "relationship_types": ["drug_treats_disease", "gene_associated_with_disease"],
  "include_attributes": true
}
```

#### Graph Statistics
Get statistics about the knowledge graph.

```http
GET /graph/statistics
```

**Response:**
```json
{
  "total_entities": 125000,
  "total_relationships": 850000,
  "entity_types": {
    "gene": 25000,
    "disease": 15000,
    "drug": 8000,
    "protein": 20000
  },
  "relationship_types": {
    "drug_treats_disease": 45000,
    "gene_associated_with_disease": 120000,
    "protein_protein_interaction": 200000
  },
  "last_updated": "2024-01-15T00:00:00Z"
}
```

### Omics Data API

#### Upload Dataset
Upload omics data for analysis.

```http
POST /omics/datasets
Content-Type: multipart/form-data
```

**Form Data:**
- `file`: Data file (CSV, TSV, Excel)
- `dataset_name`: Name for the dataset
- `data_type`: Type of omics data (genomics, transcriptomics, etc.)
- `metadata`: JSON metadata about the dataset

#### List Datasets
Get available omics datasets.

```http
GET /omics/datasets?owner={user_id}&data_type={type}
```

#### Run Analysis
Execute omics data analysis workflows.

```http
POST /omics/analyses
```

**Request Body:**
```json
{
  "dataset_id": "dataset_123",
  "analysis_type": "differential_expression",
  "parameters": {
    "comparison_groups": ["treatment", "control"],
    "p_value_threshold": 0.05,
    "fold_change_threshold": 2.0
  }
}
```

### Curation API

#### Custom Entities
Add custom entities to personal knowledge graphs.

```http
POST /curation/entities
```

**Request Body:**
```json
{
  "name": "Custom Gene X",
  "entity_type": "gene",
  "description": "A novel gene discovered in our lab",
  "synonyms": ["GeneX", "GENX"],
  "attributes": {
    "chromosome": "chr1",
    "genomic_location": "12345-67890"
  }
}
```

#### Custom Relationships
Add custom relationships between entities.

```http
POST /curation/relationships
```

#### Key Sentences
Extract and curate key sentences from literature.

```http
POST /curation/key-sentences
```

### Chat API

#### Chat Completion
Interact with AI models for question answering.

```http
POST /chat/completions
```

**Request Body:**
```json
{
  "model": "gpt-3.5-turbo",
  "messages": [
    {
      "role": "user",
      "content": "What genes are associated with Alzheimer's disease?"
    }
  ],
  "context": {
    "include_knowledge_graph": true,
    "include_literature": true
  }
}
```

## Data Models

### Entity Schema
```json
{
  "id": "string",
  "name": "string",
  "entity_type": "string",
  "description": "string",
  "synonyms": ["string"],
  "attributes": {
    "key": "value"
  },
  "created_at": "datetime",
  "updated_at": "datetime"
}
```

### Relationship Schema
```json
{
  "id": "string",
  "source_id": "string",
  "target_id": "string",
  "relation_type": "string",
  "score": "float",
  "evidence": [
    {
      "type": "string",
      "source": "string",
      "description": "string"
    }
  ],
  "attributes": {
    "key": "value"
  }
}
```

### Prediction Schema
```json
{
  "id": "string",
  "entity_id": "string",
  "predicted_entity_id": "string",
  "prediction_type": "string",
  "score": "float",
  "confidence": "string",
  "model_version": "string",
  "evidence": ["object"],
  "timestamp": "datetime"
}
```

## Error Handling

### HTTP Status Codes
- `200 OK`: Successful request
- `201 Created`: Resource created successfully
- `400 Bad Request`: Invalid request parameters
- `401 Unauthorized`: Authentication required
- `403 Forbidden`: Access denied
- `404 Not Found`: Resource not found
- `429 Too Many Requests`: Rate limit exceeded
- `500 Internal Server Error`: Server error

### Error Response Format
```json
{
  "error": {
    "code": "VALIDATION_ERROR",
    "message": "Invalid entity type specified",
    "details": {
      "field": "entity_type",
      "allowed_values": ["gene", "disease", "drug", "protein"]
    }
  }
}
```

## Rate Limiting

The API implements rate limiting to ensure fair usage:

- **Authenticated users**: 1000 requests per hour
- **Anonymous users**: 100 requests per hour
- **Bulk operations**: 10 requests per minute

Rate limit headers are included in responses:
```http
X-RateLimit-Limit: 1000
X-RateLimit-Remaining: 999
X-RateLimit-Reset: 1640995200
```

## SDKs and Client Libraries

### Python SDK
```bash
pip install biomedgps-python
```

```python
from biomedgps import Client

client = Client(api_key="your_api_key")
entities = client.search_entities(query="alzheimer", entity_type="disease")
```

### R Package
```r
install.packages("biomedgpsR")

library(biomedgpsR)
client <- biomedgps_client(api_key = "your_api_key")
entities <- search_entities(client, query = "alzheimer", entity_type = "disease")
```

### JavaScript/Node.js
```bash
npm install biomedgps-js
```

```javascript
import { BioMedGPS } from 'biomedgps-js';

const client = new BioMedGPS({ apiKey: 'your_api_key' });
const entities = await client.searchEntities({ 
  query: 'alzheimer', 
  entityType: 'disease' 
});
```

## Examples and Tutorials

### Basic Entity Search
```python
import requests

response = requests.get(
    "http://localhost:8888/entities",
    params={"query": "BRCA1", "entity_type": "gene"}
)
data = response.json()
print(f"Found {len(data['entities'])} entities")
```

### Drug Discovery Workflow
```python
# 1. Find disease entity
disease = client.search_entities(query="breast cancer", entity_type="disease")[0]

# 2. Predict drug candidates
predictions = client.predict_drugs(
    entity_id=disease['id'],
    threshold=0.8,
    limit=20
)

# 3. Get detailed information for top candidates
for pred in predictions[:5]:
    drug_details = client.get_entity(pred['drug_id'])
    print(f"{drug_details['name']}: {pred['score']}")
```

### Pathway Analysis
```python
# Get genes associated with a disease
disease_genes = client.get_relationships(
    entity_id="DOID:1612",  # breast cancer
    relation_type="gene_associated_with_disease"
)

# Extract subgraph for pathway analysis
subgraph = client.extract_subgraph(
    entities=[gene['target_id'] for gene in disease_genes[:50]],
    max_distance=2
)
```

## Webhooks

Subscribe to real-time updates for data changes and analysis completions.

### Webhook Configuration
```http
POST /webhooks
```

**Request Body:**
```json
{
  "url": "https://your-app.com/webhook",
  "events": ["entity.created", "analysis.completed"],
  "secret": "webhook_secret"
}
```

### Webhook Events
- `entity.created`: New entity added
- `entity.updated`: Entity modified  
- `relationship.created`: New relationship added
- `analysis.completed`: Omics analysis finished
- `prediction.completed`: Prediction job finished

## Support and Resources

- **Interactive API Documentation**: `/docs` endpoint
- **GitHub Repository**: [https://github.com/yjcyxky/biomedgps](https://github.com/yjcyxky/biomedgps)
- **Issue Tracker**: [GitHub Issues](https://github.com/yjcyxky/biomedgps/issues)
- **Community Forum**: [Discussions](https://github.com/yjcyxky/biomedgps/discussions)

For technical support, please create an issue on GitHub or contact the development team directly.