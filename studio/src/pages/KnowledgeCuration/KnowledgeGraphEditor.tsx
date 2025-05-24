// eslint-disable-next-line no-undef
/*global chrome*/

import React, { useEffect } from "react";
import GraphForm from "biominer-components/dist/KnowledgeGraphEditor/GraphForm";
import GraphTable from "biominer-components/dist/KnowledgeGraphEditor/GraphTable";
import KnowledgeGraph from "biominer-components/dist/KnowledgeGraph";
import { Row, Col, Tabs, Empty, Button, Space, Select, message } from "antd";
import type {
    GraphEdge,
    GraphTableData,
} from "biominer-components/dist/KnowledgeGraphEditor/index.t";
import type { GraphData } from "biominer-components/dist/typings";
import { TableOutlined, BulbOutlined, ForkOutlined } from "@ant-design/icons";
import {
    fetchNodes,
    postSubgraph,
    deleteSubgraph,
    fetchRelations,
    fetchSubgraphs,
    fetchEntity2D as fetchEntity2d,
    fetchEntityColorMap,
    fetchRelationCounts,
    fetchOneStepLinkedNodes,
    fetchEdgesAutoConnectNodes,
    fetchCuratedGraph as getGraph,
    fetchCuratedKnowledgesByOwner as getKnowledges,
    fetchStatistics as getStatistics,
    fetchEntities as getEntities,
    postCuratedKnowledge as postKnowledge,
    putCuratedKnowledge as putKnowledgeById,
    deleteCuratedKnowledge as deleteKnowledgeById,
} from "@/services/swagger/KnowledgeGraph";

import "./knowledge-graph-editor.less";

type KnowledgeGraphEditorProps = {
    curator?: string;
    activeOrg?: string;
    projectId?: string;
};

type QueryParams = {
    page: number;
    page_size: number;
    curator?: string;
    strict_mode: boolean;
    organization_id?: string;
    project_id?: string;
}

const KnowledgeGraphEditor: React.FC<KnowledgeGraphEditorProps> = (props) => {
    const [refreshKey, setRefreshKey] = React.useState<number>(0);
    const [formData, setFormData] = React.useState<GraphEdge>({} as GraphEdge);
    const [currentMode, setCurrentMode] = React.useState<string>("curator");
    const [page, setPage] = React.useState<number>(1);
    const [pageSize, setPageSize] = React.useState<number>(10);
    const [queryParams, setQueryParams] = React.useState<QueryParams>({
        strict_mode: true,
    } as QueryParams); // The query params for the getKnowledges and getGraph
    const [graphData, setGraphData] = React.useState<GraphData>({
        nodes: [],
        edges: [],
    });

    const forceUpdate = () => {
        setRefreshKey(refreshKey + 1);
    };

    useEffect(() => {
        console.log("Knowledge graph editor props: ", props);
        getGraph(queryParams)
            .then((response: any) => {
                console.log("Get graph: ", response);
                const numOfEdges = response.edges.length;

                if (numOfEdges < pageSize) {
                    // There are some edges have been filtered out, because they are Unknown:Unknown or invalid node ids.
                    message.warning("Some edges have not been shown in the graph, because they are Unknown:Unknown or invalid node ids.");
                }

                let graphData: GraphData = {
                    nodes: response.nodes,
                    edges: response.edges,
                };
                setGraphData(graphData);
            })
            .catch((error: any) => {
                console.log("Get graph error: ", error);
                message.error("Failed to get graph data.");
                setGraphData({
                    nodes: [],
                    edges: [],
                });
            });
    }, [page, pageSize, queryParams]);

    const getKnowledgesData = (
        page: number,
        pageSize: number
    ): Promise<GraphTableData> => {
        return new Promise((resolve, reject) => {
            let queryParams: QueryParams = {
                page: page,
                page_size: pageSize,
                strict_mode: true,
            };

            if (props.curator) {
                // Curator is required for the backend API
                queryParams["curator"] = props.curator;
            }

            // if (currentMode === "organization") {
            //     queryParams["organization_id"] = props.activeOrg;
            // } else if (currentMode === "project" && props.projectId) {
            //     queryParams["project_id"] = props.projectId;
            // }

            let newQueryParams: QueryParams = queryParams;

            // Use the same query params for the getKnowledges and getGraph
            setPage(page);
            setPageSize(pageSize);
            setQueryParams(newQueryParams); // Save the query params for the getGraph

            getKnowledges(newQueryParams)
                .then((response: any) => {
                    console.log("Get knowledges: ", response);
                    resolve({
                        data: response.records,
                        total: response.total,
                        page: page,
                        pageSize: pageSize,
                    });
                })
                .catch((error: any) => {
                    console.log("Get knowledges error: ", error);
                    reject(error);
                });
        });
    };

    const editKnowledge = (record: GraphEdge) => {
        console.log("Edit knowledge: ", record);
        setFormData(record);
    };

    const items = [
        {
            key: "table-viewer",
            label: (
                <span>
                    <BulbOutlined />
                    Table Viewer
                </span>
            ),
            children: (
                <GraphTable
                    yScroll={"calc(100vh - 250px)"}
                    key={refreshKey + currentMode}
                    getTableData={getKnowledgesData}
                    pageSize={pageSize}
                    pageSizeOptions={['10', '20', '50', '100', '300', '500', '1000']}
                    page={page}
                // Don'w allow to edit or delete the knowledge in a query table
                // editKnowledge={editKnowledge}
                // deleteKnowledgeById={deleteKnowledgeById}
                />
            ),
        },
        {
            key: "graph-viewer",
            label: (
                <span>
                    <ForkOutlined />
                    Graph Viewer
                </span>
            ),
            children: (
                <KnowledgeGraph
                    key={refreshKey + currentMode}
                    data={graphData}
                    apis={{
                        // @ts-ignore
                        GetEntitiesFn: getEntities,
                        GetStatisticsFn: getStatistics,
                        // @ts-ignore
                        GetRelationsFn: fetchRelations,
                        GetRelationCountsFn: fetchRelationCounts,
                        GetGraphHistoryFn: fetchSubgraphs,
                        PostGraphHistoryFn: postSubgraph,
                        DeleteGraphHistoryFn: deleteSubgraph,
                        GetNodesFn: fetchNodes,
                        GetOneStepLinkedNodesFn: fetchOneStepLinkedNodes,
                        GetConnectedNodesFn: fetchEdgesAutoConnectNodes,
                        GetEntity2DFn: fetchEntity2d,
                        GetEntityColorMapFn: fetchEntityColorMap,
                    }}
                />
            ),
            disabled: false,
        }
    ];

    const updateMode = (value: string) => {
        console.log("Update mode: ", value);
        setCurrentMode(value);
    };

    return (
        <Tabs
            className="knowledge-graph-editor"
            size="small"
            defaultActiveKey="table-viewer"
            items={items}
            tabBarExtraContent={
                <Space>
                    {
                        /* <Select
                            onChange={updateMode}
                            defaultValue="curator"
                            style={{ width: 120 }}
                        >
                            <Select.Option value="curator">Curator</Select.Option>
                            <Select.Option value="organization">Organization</Select.Option>
                            <Select.Option disabled={!props.projectId} value="project">
                                Project
                            </Select.Option>
                        </Select> */
                    }
                    <Button type="primary" onClick={() => forceUpdate()} className="knowledge-graph-editor-force-update-btn">
                        Force Update
                    </Button>
                </Space>
            }
        />
    );
};

export default KnowledgeGraphEditor;