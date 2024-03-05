// @ts-ignore
/* eslint-disable */
import { request } from 'umi';

/** Call `/api/v1/auto-connect-nodes` with query params to fetch edges which connect the input nodes. GET /api/v1/auto-connect-nodes */
export async function fetchEdgesAutoConnectNodes(
  // 叠加生成的Param类型 (非body参数swagger默认没有生成对象)
  params: API.fetchEdgesAutoConnectNodesParams,
  options?: { [key: string]: any },
) {
  return request<API.Graph>('/api/v1/auto-connect-nodes', {
    method: 'GET',
    params: {
      ...params,
    },
    ...(options || {}),
  });
}

/** Call `/api/v1/curated-graph` with query params to fetch curated graph. GET /api/v1/curated-graph */
export async function fetchCuratedGraph(
  // 叠加生成的Param类型 (非body参数swagger默认没有生成对象)
  params: API.fetchCuratedGraphParams,
  options?: { [key: string]: any },
) {
  return request<API.Graph>('/api/v1/curated-graph', {
    method: 'GET',
    params: {
      ...params,
    },
    ...(options || {}),
  });
}

/** Call `/api/v1/curated-knowledges` with query params to fetch curated knowledges. GET /api/v1/curated-knowledges */
export async function fetchCuratedKnowledges(
  // 叠加生成的Param类型 (非body参数swagger默认没有生成对象)
  params: API.fetchCuratedKnowledgesParams,
  options?: { [key: string]: any },
) {
  return request<API.RecordResponseKnowledgeCuration>('/api/v1/curated-knowledges', {
    method: 'GET',
    params: {
      ...params,
    },
    ...(options || {}),
  });
}

/** Call `/api/v1/curated-knowledges` with payload to create a curated knowledge. POST /api/v1/curated-knowledges */
export async function postCuratedKnowledge(
  body: API.KnowledgeCuration,
  options?: { [key: string]: any },
) {
  return request<API.KnowledgeCuration>('/api/v1/curated-knowledges', {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
    },
    data: body,
    ...(options || {}),
  });
}

/** Call `/api/v1/curated-knowledges-by-owner` with query params to fetch curated knowledges by owner. GET /api/v1/curated-knowledges-by-owner */
export async function fetchCuratedKnowledgesByOwner(
  // 叠加生成的Param类型 (非body参数swagger默认没有生成对象)
  params: API.fetchCuratedKnowledgesByOwnerParams,
  options?: { [key: string]: any },
) {
  return request<API.RecordResponseKnowledgeCuration>('/api/v1/curated-knowledges-by-owner', {
    method: 'GET',
    params: {
      ...params,
    },
    ...(options || {}),
  });
}

/** Call `/api/v1/curated-knowledges/:id` with payload to create a curated knowledge. PUT /api/v1/curated-knowledges/${param0} */
export async function putCuratedKnowledge(
  // 叠加生成的Param类型 (非body参数swagger默认没有生成对象)
  params: API.putCuratedKnowledgeParams,
  body: API.KnowledgeCuration,
  options?: { [key: string]: any },
) {
  const { id: param0, ...queryParams } = params;
  return request<API.KnowledgeCuration>(`/api/v1/curated-knowledges/${param0}`, {
    method: 'PUT',
    headers: {
      'Content-Type': 'application/json',
    },
    params: { ...queryParams },
    data: body,
    ...(options || {}),
  });
}

/** Call `/api/v1/curated-knowledges/:id` with payload to delete a curated knowledge. DELETE /api/v1/curated-knowledges/${param0} */
export async function deleteCuratedKnowledge(
  // 叠加生成的Param类型 (非body参数swagger默认没有生成对象)
  params: API.deleteCuratedKnowledgeParams,
  options?: { [key: string]: any },
) {
  const { id: param0, ...queryParams } = params;
  return request<any>(`/api/v1/curated-knowledges/${param0}`, {
    method: 'DELETE',
    params: { ...queryParams },
    ...(options || {}),
  });
}

/** Call `/api/v1/entities` with query params to fetch entities. GET /api/v1/entities */
export async function fetchEntities(
  // 叠加生成的Param类型 (非body参数swagger默认没有生成对象)
  params: API.fetchEntitiesParams,
  options?: { [key: string]: any },
) {
  return request<API.RecordResponseEntity>('/api/v1/entities', {
    method: 'GET',
    params: {
      ...params,
    },
    ...(options || {}),
  });
}

/** Call `/api/v1/entity-colormap` with query params to fetch all entity colormap. GET /api/v1/entity-colormap */
export async function fetchEntityColorMap(options?: { [key: string]: any }) {
  return request<Record<string, any>>('/api/v1/entity-colormap', {
    method: 'GET',
    ...(options || {}),
  });
}

/** Call `/api/v1/entity-metadata` with query params to fetch all entity metadata. GET /api/v1/entity-metadata */
export async function fetchEntityMetadata(options?: { [key: string]: any }) {
  return request<API.EntityMetadata[]>('/api/v1/entity-metadata', {
    method: 'GET',
    ...(options || {}),
  });
}

/** Call `/api/v1/entity2d` with query params to fetch entity2d. GET /api/v1/entity2d */
export async function fetchEntity2d(
  // 叠加生成的Param类型 (非body参数swagger默认没有生成对象)
  params: API.fetchEntity2dParams,
  options?: { [key: string]: any },
) {
  return request<API.RecordResponseEntity2D>('/api/v1/entity2d', {
    method: 'GET',
    params: {
      ...params,
    },
    ...(options || {}),
  });
}

/** Call `/api/v1/llm` with query params to get answer from LLM. POST /api/v1/llm */
export async function askLLM(
  // 叠加生成的Param类型 (非body参数swagger默认没有生成对象)
  params: API.askLLMParams,
  body: API.Context,
  options?: { [key: string]: any },
) {
  return request<API.LlmResponse>('/api/v1/llm', {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
    },
    params: {
      ...params,
    },
    data: body,
    ...(options || {}),
  });
}

/** Call `/api/v1/nodes` with query params to fetch nodes. GET /api/v1/nodes */
export async function fetchNodes(
  // 叠加生成的Param类型 (非body参数swagger默认没有生成对象)
  params: API.fetchNodesParams,
  options?: { [key: string]: any },
) {
  return request<API.Graph>('/api/v1/nodes', {
    method: 'GET',
    params: {
      ...params,
    },
    ...(options || {}),
  });
}

/** Call `/api/v1/one-step-linked-nodes` with query params to fetch linked nodes with one step. GET /api/v1/one-step-linked-nodes */
export async function fetchOneStepLinkedNodes(
  // 叠加生成的Param类型 (非body参数swagger默认没有生成对象)
  params: API.fetchOneStepLinkedNodesParams,
  options?: { [key: string]: any },
) {
  return request<API.Graph>('/api/v1/one-step-linked-nodes', {
    method: 'GET',
    params: {
      ...params,
    },
    ...(options || {}),
  });
}

/** Call `/api/v1/paths` with query params to fetch paths. GET /api/v1/paths */
export async function fetchPaths(
  // 叠加生成的Param类型 (非body参数swagger默认没有生成对象)
  params: API.fetchPathsParams,
  options?: { [key: string]: any },
) {
  return request<API.Graph>('/api/v1/paths', {
    method: 'GET',
    params: {
      ...params,
    },
    ...(options || {}),
  });
}

/** Call `/api/v1/predicted-nodes` with query params to fetch predicted nodes. GET /api/v1/predicted-nodes */
export async function fetchPredictedNodes(
  // 叠加生成的Param类型 (非body参数swagger默认没有生成对象)
  params: API.fetchPredictedNodesParams,
  options?: { [key: string]: any },
) {
  return request<API.Graph>('/api/v1/predicted-nodes', {
    method: 'GET',
    params: {
      ...params,
    },
    ...(options || {}),
  });
}

/** Call `/api/v1/relation-counts` with query params to fetch relation counts. GET /api/v1/relation-counts */
export async function fetchRelationCounts(
  // 叠加生成的Param类型 (非body参数swagger默认没有生成对象)
  params: API.fetchRelationCountsParams,
  options?: { [key: string]: any },
) {
  return request<API.RelationCount[]>('/api/v1/relation-counts', {
    method: 'GET',
    params: {
      ...params,
    },
    ...(options || {}),
  });
}

/** Call `/api/v1/relation-metadata` with query params to fetch all relation metadata. GET /api/v1/relation-metadata */
export async function fetchRelationMetadata(options?: { [key: string]: any }) {
  return request<API.RelationMetadata[]>('/api/v1/relation-metadata', {
    method: 'GET',
    ...(options || {}),
  });
}

/** Call `/api/v1/relations` with query params to fetch relations. GET /api/v1/relations */
export async function fetchRelations(
  // 叠加生成的Param类型 (非body参数swagger默认没有生成对象)
  params: API.fetchRelationsParams,
  options?: { [key: string]: any },
) {
  return request<API.RecordResponseRelation>('/api/v1/relations', {
    method: 'GET',
    params: {
      ...params,
    },
    ...(options || {}),
  });
}

/** Call `/api/v1/shared-nodes` with query params to fetch shared nodes. GET /api/v1/shared-nodes */
export async function fetchSharedNodes(
  // 叠加生成的Param类型 (非body参数swagger默认没有生成对象)
  params: API.fetchSharedNodesParams,
  options?: { [key: string]: any },
) {
  return request<API.Graph>('/api/v1/shared-nodes', {
    method: 'GET',
    params: {
      ...params,
    },
    ...(options || {}),
  });
}

/** Call `/api/v1/statistics` with query params to fetch all entity & relation metadata. GET /api/v1/statistics */
export async function fetchStatistics(options?: { [key: string]: any }) {
  return request<API.Statistics>('/api/v1/statistics', {
    method: 'GET',
    ...(options || {}),
  });
}

/** Call `/api/v1/subgraphs` with query params to fetch subgraphs. GET /api/v1/subgraphs */
export async function fetchSubgraphs(
  // 叠加生成的Param类型 (非body参数swagger默认没有生成对象)
  params: API.fetchSubgraphsParams,
  options?: { [key: string]: any },
) {
  return request<API.RecordResponseSubgraph>('/api/v1/subgraphs', {
    method: 'GET',
    params: {
      ...params,
    },
    ...(options || {}),
  });
}

/** Call `/api/v1/subgraphs` with payload to create a subgraph. POST /api/v1/subgraphs */
export async function postSubgraph(body: API.Subgraph, options?: { [key: string]: any }) {
  return request<API.Subgraph>('/api/v1/subgraphs', {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
    },
    data: body,
    ...(options || {}),
  });
}

/** Call `/api/v1/subgraphs/:id` with payload to update a subgraph. PUT /api/v1/subgraphs/${param0} */
export async function putSubgraph(
  // 叠加生成的Param类型 (非body参数swagger默认没有生成对象)
  params: API.putSubgraphParams,
  body: API.Subgraph,
  options?: { [key: string]: any },
) {
  const { id: param0, ...queryParams } = params;
  return request<API.Subgraph>(`/api/v1/subgraphs/${param0}`, {
    method: 'PUT',
    headers: {
      'Content-Type': 'application/json',
    },
    params: { ...queryParams },
    data: body,
    ...(options || {}),
  });
}

/** Call `/api/v1/subgraphs/:id` with payload to create subgraph. DELETE /api/v1/subgraphs/${param0} */
export async function deleteSubgraph(
  // 叠加生成的Param类型 (非body参数swagger默认没有生成对象)
  params: API.deleteSubgraphParams,
  options?: { [key: string]: any },
) {
  const { id: param0, ...queryParams } = params;
  return request<any>(`/api/v1/subgraphs/${param0}`, {
    method: 'DELETE',
    params: { ...queryParams },
    ...(options || {}),
  });
}
