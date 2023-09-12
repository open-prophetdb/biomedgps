### Example1

```json
{
  "operator": "AND",
  "items": [
    {
      "operator": "OR",
      "items": [
        {
          "operator": "ilike",
          "field": "name",
          "value": "%CFS%"
        },
        {
          "operator": "ilike",
          "field": "id",
          "value": "%CFS%"
        },
        {
          "operator": "ilike",
          "field": "synonyms",
          "value": "%CFS%"
        },
        {
          "operator": "ilike",
          "field": "xrefs",
          "value": "%CFS%"
        }
      ]
    },
    {
      "operator": "=",
      "field": "label",
      "value": "Disease"
    }
  ]
}
```

### Example2

```json
{
  "operator": "or",
  "items": [
    {
      "operator": "ilike",
      "field": "name",
      "value": "%CFS%"
    },
    {
      "operator": "ilike",
      "field": "id",
      "value": "%CFS%"
    }
  ]
}
```
