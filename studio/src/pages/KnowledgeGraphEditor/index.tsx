import React from 'react';
import { KnowledgeGraphEditor } from 'biominer-components';
import {
  fetchCuratedKnowledges, fetchStatistics, fetchEntities, postCuratedKnowledge, putCuratedKnowledge, deleteCuratedKnowledge
} from '@/services/swagger/KnowledgeGraph';

import './index.less'

const KnowledgeGraphEditorWrapper: React.FC = () => {
  return <KnowledgeGraphEditor
    getKnowledges={fetchCuratedKnowledges}
    getStatistics={fetchStatistics}
    getEntities={fetchEntities}
    postKnowledge={postCuratedKnowledge}
    putKnowledgeById={putCuratedKnowledge}
    deleteKnowledgeById={deleteCuratedKnowledge}
  />;
};

export default KnowledgeGraphEditorWrapper;
