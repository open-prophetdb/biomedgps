// @ts-ignore
/* eslint-disable */
import { request } from 'umi';

/** Call `/api/v1/auto-connect-nodes` with query params to fetch edges which connect the input nodes. GET /api/v1/auto-connect-nodes */
export async function fetchEdgesAutoConnectNodes(
  // 叠加生成的Param类型 (非body参数swagger默认没有生成对象)
  params: swagger.fetchEdgesAutoConnectNodesParams,
  options?: { [key: string]: any },
) {
  return request<swagger.Graph>('/api/v1/auto-connect-nodes', {
    method: 'GET',
    params: {
      ...params,
    },
    ...(options || {}),
  });
}

/** Call `/api/v1/configurations` with query params to fetch configurations. GET /api/v1/configurations */
export async function fetchConfigurations(
  // 叠加生成的Param类型 (非body参数swagger默认没有生成对象)
  params: swagger.fetchConfigurationsParams,
  options?: { [key: string]: any },
) {
  return request<swagger.RecordResponseConfiguration>('/api/v1/configurations', {
    method: 'GET',
    params: {
      ...params,
    },
    ...(options || {}),
  });
}

/** Call `/api/v1/configurations` with payload to create a configuration. POST /api/v1/configurations */
export async function postConfiguration(
  body: swagger.Configuration,
  options?: { [key: string]: any },
) {
  return request<swagger.Configuration>('/api/v1/configurations', {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
    },
    data: body,
    ...(options || {}),
  });
}

/** Call `/api/v1/configurations` with payload to delete a configuration. DELETE /api/v1/configurations */
export async function deleteConfiguration(
  // 叠加生成的Param类型 (非body参数swagger默认没有生成对象)
  params: swagger.deleteConfigurationParams,
  options?: { [key: string]: any },
) {
  return request<any>('/api/v1/configurations', {
    method: 'DELETE',
    params: {
      ...params,
    },
    ...(options || {}),
  });
}

/** Call `/api/v1/configurations/:id` with payload to update a configuration. PUT /api/v1/configurations/${param0} */
export async function putConfiguration(
  // 叠加生成的Param类型 (非body参数swagger默认没有生成对象)
  params: swagger.putConfigurationParams,
  body: swagger.Configuration,
  options?: { [key: string]: any },
) {
  const { id: param0, ...queryParams } = params;
  return request<swagger.Configuration>(`/api/v1/configurations/${param0}`, {
    method: 'PUT',
    headers: {
      'Content-Type': 'application/json',
    },
    params: { ...queryParams },
    data: body,
    ...(options || {}),
  });
}

/** Call `/api/v1/curated-graph` with query params to fetch curated graph. GET /api/v1/curated-graph */
export async function fetchCuratedGraph(
  // 叠加生成的Param类型 (非body参数swagger默认没有生成对象)
  params: swagger.fetchCuratedGraphParams,
  options?: { [key: string]: any },
) {
  return request<swagger.Graph>('/api/v1/curated-graph', {
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
  params: swagger.fetchCuratedKnowledgesParams,
  options?: { [key: string]: any },
) {
  return request<swagger.RecordResponseKnowledgeCuration>('/api/v1/curated-knowledges', {
    method: 'GET',
    params: {
      ...params,
    },
    ...(options || {}),
  });
}

/** Call `/api/v1/curated-knowledges` with payload to create a curated knowledge. POST /api/v1/curated-knowledges */
export async function postCuratedKnowledge(
  body: swagger.KnowledgeCuration,
  options?: { [key: string]: any },
) {
  return request<swagger.KnowledgeCuration>('/api/v1/curated-knowledges', {
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
  params: swagger.fetchCuratedKnowledgesByOwnerParams,
  options?: { [key: string]: any },
) {
  return request<swagger.RecordResponseKnowledgeCuration>('/api/v1/curated-knowledges-by-owner', {
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
  params: swagger.putCuratedKnowledgeParams,
  body: swagger.KnowledgeCuration,
  options?: { [key: string]: any },
) {
  const { id: param0, ...queryParams } = params;
  return request<swagger.KnowledgeCuration>(`/api/v1/curated-knowledges/${param0}`, {
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
  params: swagger.deleteCuratedKnowledgeParams,
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
  params: swagger.fetchEntitiesParams,
  options?: { [key: string]: any },
) {
  return request<swagger.RecordResponseEntity>('/api/v1/entities', {
    method: 'GET',
    params: {
      ...params,
    },
    ...(options || {}),
  });
}

/** Call `/api/v1/entity-attr` with query params to fetch all entity attributes. GET /api/v1/entity-attr */
export async function fetchEntityAttributes(
  // 叠加生成的Param类型 (非body参数swagger默认没有生成对象)
  params: swagger.fetchEntityAttributesParams,
  options?: { [key: string]: any },
) {
  return request<swagger.EntityAttr>('/api/v1/entity-attr', {
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

/** Call `/api/v1/entity-curations` with query params to fetch entity curations. GET /api/v1/entity-curations */
export async function fetchEntityCuration(
  // 叠加生成的Param类型 (非body参数swagger默认没有生成对象)
  params: swagger.fetchEntityCurationParams,
  options?: { [key: string]: any },
) {
  return request<swagger.RecordResponseEntityCuration>('/api/v1/entity-curations', {
    method: 'GET',
    params: {
      ...params,
    },
    ...(options || {}),
  });
}

/** Call `/api/v1/entity-curations` with payload to create a entity curation. POST /api/v1/entity-curations */
export async function postEntityCuration(
  body: swagger.EntityCuration,
  options?: { [key: string]: any },
) {
  return request<swagger.EntityCuration>('/api/v1/entity-curations', {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
    },
    data: body,
    ...(options || {}),
  });
}

/** Call `/api/v1/entity-curations` with payload to delete a entity curation. DELETE /api/v1/entity-curations */
export async function deleteEntityCurationRecord(
  // 叠加生成的Param类型 (非body参数swagger默认没有生成对象)
  params: swagger.deleteEntityCurationRecordParams,
  options?: { [key: string]: any },
) {
  return request<any>('/api/v1/entity-curations', {
    method: 'DELETE',
    params: {
      ...params,
    },
    ...(options || {}),
  });
}

/** Call `/api/v1/entity-curations-by-owner` with query params to fetch entity curations by owner. GET /api/v1/entity-curations-by-owner */
export async function fetchEntityCurationByOwner(
  // 叠加生成的Param类型 (非body参数swagger默认没有生成对象)
  params: swagger.fetchEntityCurationByOwnerParams,
  options?: { [key: string]: any },
) {
  return request<swagger.RecordResponseEntityCuration>('/api/v1/entity-curations-by-owner', {
    method: 'GET',
    params: {
      ...params,
    },
    ...(options || {}),
  });
}

/** Call `/api/v1/entity-curations/:id` with payload to update a entity curation. PUT /api/v1/entity-curations/${param0} */
export async function putEntityCuration(
  // 叠加生成的Param类型 (非body参数swagger默认没有生成对象)
  params: swagger.putEntityCurationParams,
  body: swagger.EntityCuration,
  options?: { [key: string]: any },
) {
  const { id: param0, ...queryParams } = params;
  return request<swagger.EntityCuration>(`/api/v1/entity-curations/${param0}`, {
    method: 'PUT',
    headers: {
      'Content-Type': 'application/json',
    },
    params: { ...queryParams },
    data: body,
    ...(options || {}),
  });
}

/** Call `/api/v1/entity-curations/:id` with payload to delete a entity curation. DELETE /api/v1/entity-curations/${param0} */
export async function deleteEntityCuration(
  // 叠加生成的Param类型 (非body参数swagger默认没有生成对象)
  params: swagger.deleteEntityCurationParams,
  options?: { [key: string]: any },
) {
  const { id: param0, ...queryParams } = params;
  return request<any>(`/api/v1/entity-curations/${param0}`, {
    method: 'DELETE',
    params: { ...queryParams },
    ...(options || {}),
  });
}

/** Call `/api/v1/entity-metadata` with query params to fetch all entity metadata. GET /api/v1/entity-metadata */
export async function fetchEntityMetadata(options?: { [key: string]: any }) {
  return request<swagger.EntityMetadata[]>('/api/v1/entity-metadata', {
    method: 'GET',
    ...(options || {}),
  });
}

/** Call `/api/v1/entity-metadata-curations` with query params to fetch entity metadata curations. GET /api/v1/entity-metadata-curations */
export async function fetchEntityMetadataCuration(
  // 叠加生成的Param类型 (非body参数swagger默认没有生成对象)
  params: swagger.fetchEntityMetadataCurationParams,
  options?: { [key: string]: any },
) {
  return request<swagger.RecordResponseEntityMetadataCuration>(
    '/api/v1/entity-metadata-curations',
    {
      method: 'GET',
      params: {
        ...params,
      },
      ...(options || {}),
    },
  );
}

/** Call `/api/v1/entity-metadata-curations` with payload to create a entity metadata curation. POST /api/v1/entity-metadata-curations */
export async function postEntityMetadataCuration(
  body: swagger.EntityMetadataCuration,
  options?: { [key: string]: any },
) {
  return request<swagger.EntityMetadataCuration>('/api/v1/entity-metadata-curations', {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
    },
    data: body,
    ...(options || {}),
  });
}

/** Call `/api/v1/entity-metadata-curations` with payload to delete a entity metadata curation. DELETE /api/v1/entity-metadata-curations */
export async function deleteEntityMetadataCurationRecord(
  // 叠加生成的Param类型 (非body参数swagger默认没有生成对象)
  params: swagger.deleteEntityMetadataCurationRecordParams,
  options?: { [key: string]: any },
) {
  return request<any>('/api/v1/entity-metadata-curations', {
    method: 'DELETE',
    params: {
      ...params,
    },
    ...(options || {}),
  });
}

/** Call `/api/v1/entity-metadata-curations-by-owner` with query params to fetch entity metadata curations by owner. GET /api/v1/entity-metadata-curations-by-owner */
export async function fetchEntityMetadataCurationByOwner(
  // 叠加生成的Param类型 (非body参数swagger默认没有生成对象)
  params: swagger.fetchEntityMetadataCurationByOwnerParams,
  options?: { [key: string]: any },
) {
  return request<swagger.RecordResponseEntityMetadataCuration>(
    '/api/v1/entity-metadata-curations-by-owner',
    {
      method: 'GET',
      params: {
        ...params,
      },
      ...(options || {}),
    },
  );
}

/** Call `/api/v1/entity-metadata-curations/:id` with payload to update a entity metadata curation. PUT /api/v1/entity-metadata-curations/${param0} */
export async function putEntityMetadataCuration(
  // 叠加生成的Param类型 (非body参数swagger默认没有生成对象)
  params: swagger.putEntityMetadataCurationParams,
  body: swagger.EntityMetadataCuration,
  options?: { [key: string]: any },
) {
  const { id: param0, ...queryParams } = params;
  return request<swagger.EntityMetadataCuration>(`/api/v1/entity-metadata-curations/${param0}`, {
    method: 'PUT',
    headers: {
      'Content-Type': 'application/json',
    },
    params: { ...queryParams },
    data: body,
    ...(options || {}),
  });
}

/** Call `/api/v1/entity-metadata-curations/:id` with payload to delete a entity metadata curation. DELETE /api/v1/entity-metadata-curations/${param0} */
export async function deleteEntityMetadataCuration(
  // 叠加生成的Param类型 (非body参数swagger默认没有生成对象)
  params: swagger.deleteEntityMetadataCurationParams,
  options?: { [key: string]: any },
) {
  const { id: param0, ...queryParams } = params;
  return request<any>(`/api/v1/entity-metadata-curations/${param0}`, {
    method: 'DELETE',
    params: { ...queryParams },
    ...(options || {}),
  });
}

/** Call `/api/v1/entity2d` with query params to fetch entity2d. GET /api/v1/entity2d */
export async function fetchEntity2D(
  // 叠加生成的Param类型 (非body参数swagger默认没有生成对象)
  params: swagger.fetchEntity2DParams,
  options?: { [key: string]: any },
) {
  return request<swagger.RecordResponseEntity2D>('/api/v1/entity2d', {
    method: 'GET',
    params: {
      ...params,
    },
    ...(options || {}),
  });
}

/** Call `/api/v1/key-sentence-curations` with query params to fetch key sentence curations. GET /api/v1/key-sentence-curations */
export async function fetchKeySentenceCuration(
  // 叠加生成的Param类型 (非body参数swagger默认没有生成对象)
  params: swagger.fetchKeySentenceCurationParams,
  options?: { [key: string]: any },
) {
  return request<swagger.RecordResponseKeySentenceCuration>('/api/v1/key-sentence-curations', {
    method: 'GET',
    params: {
      ...params,
    },
    ...(options || {}),
  });
}

/** Call `/api/v1/key-sentence-curations` with payload to create a key sentence curation. POST /api/v1/key-sentence-curations */
export async function postKeySentenceCuration(
  body: swagger.KeySentenceCuration,
  options?: { [key: string]: any },
) {
  return request<swagger.KeySentenceCuration>('/api/v1/key-sentence-curations', {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
    },
    data: body,
    ...(options || {}),
  });
}

/** Call `/api/v1/key-sentence-curations` with payload to delete a key sentence curation. DELETE /api/v1/key-sentence-curations */
export async function deleteKeySentenceCurationByFingerprint(
  // 叠加生成的Param类型 (非body参数swagger默认没有生成对象)
  params: swagger.deleteKeySentenceCurationByFingerprintParams,
  options?: { [key: string]: any },
) {
  return request<any>('/api/v1/key-sentence-curations', {
    method: 'DELETE',
    params: {
      ...params,
    },
    ...(options || {}),
  });
}

/** Call `/api/v1/key-sentence-curations-by-owner` with query params to fetch key sentence curations by owner. GET /api/v1/key-sentence-curations-by-owner */
export async function fetchKeySentenceCurationByOwner(
  // 叠加生成的Param类型 (非body参数swagger默认没有生成对象)
  params: swagger.fetchKeySentenceCurationByOwnerParams,
  options?: { [key: string]: any },
) {
  return request<swagger.RecordResponseKeySentenceCuration>(
    '/api/v1/key-sentence-curations-by-owner',
    {
      method: 'GET',
      params: {
        ...params,
      },
      ...(options || {}),
    },
  );
}

/** Call `/api/v1/key-sentence-curations/:id` with payload to update a key sentence curation. PUT /api/v1/key-sentence-curations/${param0} */
export async function putKeySentenceCuration(
  // 叠加生成的Param类型 (非body参数swagger默认没有生成对象)
  params: swagger.putKeySentenceCurationParams,
  body: swagger.KeySentenceCuration,
  options?: { [key: string]: any },
) {
  const { id: param0, ...queryParams } = params;
  return request<swagger.KeySentenceCuration>(`/api/v1/key-sentence-curations/${param0}`, {
    method: 'PUT',
    headers: {
      'Content-Type': 'application/json',
    },
    params: { ...queryParams },
    data: body,
    ...(options || {}),
  });
}

/** Call `/api/v1/key-sentence-curations/:id` with payload to delete a key sentence curation. DELETE /api/v1/key-sentence-curations/${param0} */
export async function deleteKeySentenceCuration(
  // 叠加生成的Param类型 (非body参数swagger默认没有生成对象)
  params: swagger.deleteKeySentenceCurationParams,
  options?: { [key: string]: any },
) {
  const { id: param0, ...queryParams } = params;
  return request<any>(`/api/v1/key-sentence-curations/${param0}`, {
    method: 'DELETE',
    params: { ...queryParams },
    ...(options || {}),
  });
}

/** Call `/api/v1/key-sentence-curations/:id/images` with payload to add an image to a key sentence curation. POST /api/v1/key-sentence-curations/${param0}/images */
export async function postKeySentenceCurationImage(
  // 叠加生成的Param类型 (非body参数swagger默认没有生成对象)
  params: swagger.postKeySentenceCurationImageParams,
  body: {
    raw_image_url: string;
    raw_image_src: string;
    name: string;
  },
  image?: File,
  options?: { [key: string]: any },
) {
  const { id: param0, ...queryParams } = params;
  const formData = new FormData();

  if (image) {
    formData.append('image', image);
  }

  Object.keys(body).forEach((ele) => {
    const item = (body as any)[ele];

    if (item !== undefined && item !== null) {
      if (typeof item === 'object' && !(item instanceof File)) {
        if (item instanceof Array) {
          item.forEach((f) => formData.append(ele, f || ''));
        } else {
          formData.append(ele, JSON.stringify(item));
        }
      } else {
        formData.append(ele, item);
      }
    }
  });

  return request<swagger.KeySentenceCuration>(`/api/v1/key-sentence-curations/${param0}/images`, {
    method: 'POST',
    params: { ...queryParams },
    data: formData,
    requestType: 'form',
    ...(options || {}),
  });
}

/** Call `/api/v1/llm` with query params to get answer from LLM. POST /api/v1/llm */
export async function askLlm(
  // 叠加生成的Param类型 (非body参数swagger默认没有生成对象)
  params: swagger.askLLMParams,
  body: swagger.Context,
  options?: { [key: string]: any },
) {
  return request<swagger.LlmResponse>('/api/v1/llm', {
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

/** Call `/api/v1/llm-prompts` with query params to get prompt templates. GET /api/v1/llm-prompts */
export async function fetchPrompts(options?: { [key: string]: any }) {
  return request<swagger.PromptList>('/api/v1/llm-prompts', {
    method: 'GET',
    ...(options || {}),
  });
}

/** Call `/api/v1/nodes` with query params to fetch nodes. GET /api/v1/nodes */
export async function fetchNodes(
  // 叠加生成的Param类型 (非body参数swagger默认没有生成对象)
  params: swagger.fetchNodesParams,
  options?: { [key: string]: any },
) {
  return request<swagger.Graph>('/api/v1/nodes', {
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
  params: swagger.fetchOneStepLinkedNodesParams,
  options?: { [key: string]: any },
) {
  return request<swagger.Graph>('/api/v1/one-step-linked-nodes', {
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
  params: swagger.fetchPathsParams,
  options?: { [key: string]: any },
) {
  return request<swagger.Graph>('/api/v1/paths', {
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
  params: swagger.fetchPredictedNodesParams,
  options?: { [key: string]: any },
) {
  return request<swagger.Graph>('/api/v1/predicted-nodes', {
    method: 'GET',
    params: {
      ...params,
    },
    ...(options || {}),
  });
}

/** Call `/api/v1/publications` with query params to fetch publications. GET /api/v1/publications */
export async function fetchPublications(
  // 叠加生成的Param类型 (非body参数swagger默认没有生成对象)
  params: swagger.fetchPublicationsParams,
  options?: { [key: string]: any },
) {
  return request<swagger.PublicationRecords>('/api/v1/publications', {
    method: 'GET',
    params: {
      ...params,
    },
    ...(options || {}),
  });
}

/** Call `/api/v1/publications-consensus` with query params to fetch publication consensus. GET /api/v1/publications-consensus/${param0} */
export async function fetchPublicationsConsensus(
  // 叠加生成的Param类型 (非body参数swagger默认没有生成对象)
  params: swagger.fetchPublicationsConsensusParams,
  options?: { [key: string]: any },
) {
  const { search_id: param0, ...queryParams } = params;
  return request<swagger.ConsensusResult>(`/api/v1/publications-consensus/${param0}`, {
    method: 'GET',
    params: { ...queryParams },
    ...(options || {}),
  });
}

/** Call `/api/v1/publications-summary` with query params to fetch publication summary. POST /api/v1/publications-summary */
export async function answerQuestionWithPublications(
  // 叠加生成的Param类型 (非body参数swagger默认没有生成对象)
  params: swagger.answerQuestionWithPublicationsParams,
  body: swagger.Publication[],
  options?: { [key: string]: any },
) {
  return request<swagger.PublicationsSummary>('/api/v1/publications-summary', {
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

/** Call `/api/v1/publications-summary` with query params to fetch publication summary. GET /api/v1/publications-summary/${param0} */
export async function fetchPublicationsSummary(
  // 叠加生成的Param类型 (非body参数swagger默认没有生成对象)
  params: swagger.fetchPublicationsSummaryParams,
  options?: { [key: string]: any },
) {
  const { search_id: param0, ...queryParams } = params;
  return request<swagger.PublicationsSummary>(`/api/v1/publications-summary/${param0}`, {
    method: 'GET',
    params: { ...queryParams },
    ...(options || {}),
  });
}

/** Call `/api/v1/publications/:id` to fetch a publication. GET /api/v1/publications/${param0} */
export async function fetchPublication(
  // 叠加生成的Param类型 (非body参数swagger默认没有生成对象)
  params: swagger.fetchPublicationParams,
  options?: { [key: string]: any },
) {
  const { id: param0, ...queryParams } = params;
  return request<swagger.Publication>(`/api/v1/publications/${param0}`, {
    method: 'GET',
    params: { ...queryParams },
    ...(options || {}),
  });
}

/** Call `/api/v1/relation-counts` with query params to fetch relation counts. GET /api/v1/relation-counts */
export async function fetchRelationCounts(
  // 叠加生成的Param类型 (非body参数swagger默认没有生成对象)
  params: swagger.fetchRelationCountsParams,
  options?: { [key: string]: any },
) {
  return request<swagger.RelationCount[]>('/api/v1/relation-counts', {
    method: 'GET',
    params: {
      ...params,
    },
    ...(options || {}),
  });
}

/** Call `/api/v1/relation-metadata` with query params to fetch all relation metadata. GET /api/v1/relation-metadata */
export async function fetchRelationMetadata(options?: { [key: string]: any }) {
  return request<swagger.RelationMetadata[]>('/api/v1/relation-metadata', {
    method: 'GET',
    ...(options || {}),
  });
}

/** Call `/api/v1/relations` with query params to fetch relations. GET /api/v1/relations */
export async function fetchRelations(
  // 叠加生成的Param类型 (非body参数swagger默认没有生成对象)
  params: swagger.fetchRelationsParams,
  options?: { [key: string]: any },
) {
  return request<swagger.RecordResponseRelation>('/api/v1/relations', {
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
  params: swagger.fetchSharedNodesParams,
  options?: { [key: string]: any },
) {
  return request<swagger.Graph>('/api/v1/shared-nodes', {
    method: 'GET',
    params: {
      ...params,
    },
    ...(options || {}),
  });
}

/** Call `/api/v1/statistics` with query params to fetch all entity & relation metadata. GET /api/v1/statistics */
export async function fetchStatistics(options?: { [key: string]: any }) {
  return request<swagger.Statistics>('/api/v1/statistics', {
    method: 'GET',
    ...(options || {}),
  });
}

/** Call `/api/v1/subgraphs` with query params to fetch subgraphs. GET /api/v1/subgraphs */
export async function fetchSubgraphs(
  // 叠加生成的Param类型 (非body参数swagger默认没有生成对象)
  params: swagger.fetchSubgraphsParams,
  options?: { [key: string]: any },
) {
  return request<swagger.RecordResponseSubgraph>('/api/v1/subgraphs', {
    method: 'GET',
    params: {
      ...params,
    },
    ...(options || {}),
  });
}

/** Call `/api/v1/subgraphs` with payload to create a subgraph. POST /api/v1/subgraphs */
export async function postSubgraph(body: swagger.Subgraph, options?: { [key: string]: any }) {
  return request<swagger.Subgraph>('/api/v1/subgraphs', {
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
  params: swagger.putSubgraphParams,
  body: swagger.Subgraph,
  options?: { [key: string]: any },
) {
  const { id: param0, ...queryParams } = params;
  return request<swagger.Subgraph>(`/api/v1/subgraphs/${param0}`, {
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
  params: swagger.deleteSubgraphParams,
  options?: { [key: string]: any },
) {
  const { id: param0, ...queryParams } = params;
  return request<any>(`/api/v1/subgraphs/${param0}`, {
    method: 'DELETE',
    params: { ...queryParams },
    ...(options || {}),
  });
}

/** Call `/api/v1/webpage-metadata` with query params to fetch webpage metadata. GET /api/v1/webpage-metadata */
export async function fetchWebpageMetadata(
  // 叠加生成的Param类型 (非body参数swagger默认没有生成对象)
  params: swagger.fetchWebpageMetadataParams,
  options?: { [key: string]: any },
) {
  return request<swagger.RecordResponseWebpageMetadata>('/api/v1/webpage-metadata', {
    method: 'GET',
    params: {
      ...params,
    },
    ...(options || {}),
  });
}

/** Call `/api/v1/webpage-metadata` with payload to create a webpage metadata. POST /api/v1/webpage-metadata */
export async function postWebpageMetadata(
  body: swagger.WebpageMetadata,
  options?: { [key: string]: any },
) {
  return request<swagger.WebpageMetadata>('/api/v1/webpage-metadata', {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
    },
    data: body,
    ...(options || {}),
  });
}

/** Call `/api/v1/webpage-metadata` with payload to delete a webpage metadata. DELETE /api/v1/webpage-metadata */
export async function deleteWebpageMetadataByFingerprint(
  // 叠加生成的Param类型 (非body参数swagger默认没有生成对象)
  params: swagger.deleteWebpageMetadataByFingerprintParams,
  options?: { [key: string]: any },
) {
  return request<any>('/api/v1/webpage-metadata', {
    method: 'DELETE',
    params: {
      ...params,
    },
    ...(options || {}),
  });
}

/** Call `/api/v1/webpage-metadata/:id` with payload to update a webpage metadata. PUT /api/v1/webpage-metadata/${param0} */
export async function putWebpageMetadata(
  // 叠加生成的Param类型 (非body参数swagger默认没有生成对象)
  params: swagger.putWebpageMetadataParams,
  body: swagger.WebpageMetadata,
  options?: { [key: string]: any },
) {
  const { id: param0, ...queryParams } = params;
  return request<swagger.WebpageMetadata>(`/api/v1/webpage-metadata/${param0}`, {
    method: 'PUT',
    headers: {
      'Content-Type': 'application/json',
    },
    params: { ...queryParams },
    data: body,
    ...(options || {}),
  });
}

/** Call `/api/v1/webpage-metadata/:id` with payload to delete a webpage metadata. DELETE /api/v1/webpage-metadata/${param0} */
export async function deleteWebpageMetadata(
  // 叠加生成的Param类型 (非body参数swagger默认没有生成对象)
  params: swagger.deleteWebpageMetadataParams,
  options?: { [key: string]: any },
) {
  const { id: param0, ...queryParams } = params;
  return request<any>(`/api/v1/webpage-metadata/${param0}`, {
    method: 'DELETE',
    params: { ...queryParams },
    ...(options || {}),
  });
}
