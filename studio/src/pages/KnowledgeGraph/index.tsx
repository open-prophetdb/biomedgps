import { isAuthenticated, logoutWithRedirect } from '@/components/util';
import { Row, Col, Button, message as AntMessage, Empty } from 'antd';
import { KnowledgeGraph } from 'biominer-components';
import React, { useEffect, useState, memo, Suspense } from 'react';
// TODO: KeepAlive will cause some bugs, so we disable it for now.
// import { KeepAlive } from 'umi';
import { MessageFilled, MessageOutlined } from '@ant-design/icons';
import {
  fetchEdgesAutoConnectNodes, fetchEntities, fetchEntity2D, fetchEntityColorMap, fetchOneStepLinkedNodes, fetchRelationCounts, fetchStatistics, fetchSubgraphs, fetchPredictedNodes, fetchNodes, fetchRelations, postSubgraph, deleteSubgraph, fetchPaths, askLlm, fetchSharedNodes, fetchPrompts as fetchLlmPrompts
} from '@/services/swagger/KnowledgeGraph';
import NodeInfoPanel from '@/NodeInfoPanel';
import EdgeInfoPanel from '@/EdgeInfoPanel';


import './index.less';

const kgFullSpan = 24;
const kgThreeQuartersSpan = 16;

const KnowledgeGraphWithChatBot: React.FC = () => {
  const [message, setMessage] = useState<string>('')
  const [chatBoxVisible, setChatBoxVisible] = useState<boolean>(false)
  const [span, setSpan] = useState<number>(kgFullSpan)
  const ChatBox = React.lazy(() => import('@/components/ChatBox'));

  useEffect(() => {
    console.log("isAuthenticated in KnowledgeGraph: ", isAuthenticated());
    if (!isAuthenticated()) {
      logoutWithRedirect();
    }
  }, [])

  useEffect(() => {
    if (chatBoxVisible) {
      setSpan(kgThreeQuartersSpan)
    } else {
      setSpan(kgFullSpan)
    }
  }, [chatBoxVisible])

  return isAuthenticated() && <Row gutter={8} className="chat-ai-container">
    {
      chatBoxVisible ? (
        <Col xxl={24 - span} xl={24 - span} lg={24 - span} md={24} sm={24} xs={24}>
          <Suspense fallback={
            <Empty description="Loading Chatbot..." />
          }>
            <ChatBox message={message}></ChatBox>
          </Suspense>
        </Col>
      ) : null
    }
    <Col xxl={span} xl={span} lg={span} md={24} sm={24} xs={24}>
      <Button shape="default" className="chat-button" onClick={() => {
        if (chatBoxVisible) {
          // Clear the message when chatbot is closed, otherwise it will activate the chat ai again when chatbot is opened.
          setMessage('')
        }
        setChatBoxVisible(!chatBoxVisible)
      }} icon={chatBoxVisible ? <MessageOutlined /> : <MessageFilled />}>
        {chatBoxVisible ? 'Hide Chatbot' : 'Show Chatbot'}
      </Button>
      <KnowledgeGraph
        apis={{
          GetStatisticsFn: fetchStatistics,
          // @ts-ignore, it doesn't matter, maybe we can fix this later.
          GetEntitiesFn: fetchEntities,
          // @ts-ignore, it doesn't matter, maybe we can fix this later.
          GetRelationsFn: fetchRelations,
          GetRelationCountsFn: fetchRelationCounts,
          GetGraphHistoryFn: fetchSubgraphs,
          PostGraphHistoryFn: postSubgraph,
          DeleteGraphHistoryFn: deleteSubgraph,
          GetNodesFn: fetchNodes,
          GetPredictedNodesFn: fetchPredictedNodes,
          GetOneStepLinkedNodesFn: fetchOneStepLinkedNodes,
          GetConnectedNodesFn: fetchEdgesAutoConnectNodes,
          GetEntity2DFn: fetchEntity2D,
          GetEntityColorMapFn: fetchEntityColorMap,
          GetNStepsLinkedNodesFn: fetchPaths,
          // @ts-ignore, it doesn't matter, maybe we can fix this later.
          AskLlmFn: askLlm,
          GetSharedNodesFn: fetchSharedNodes,
          // @ts-ignore, it seems that we don't need to fix this.
          GetPromptsFn: fetchLlmPrompts,
        }}
        NodeInfoPanel={NodeInfoPanel}
        EdgeInfoPanel={EdgeInfoPanel}
        postMessage={(message: string) => {
          if (chatBoxVisible) {
            setMessage(message)
          } else {
            AntMessage.warning('Please open the chatbot first.')
          }
        }}>
      </KnowledgeGraph>
    </Col>
  </Row>
}

export default memo(KnowledgeGraphWithChatBot);
