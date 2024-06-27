import React, { useEffect, useState } from 'react';
import type { RelationStat, EntityStat } from 'biominer-components/dist/typings';
import StatisticsChart from 'biominer-components/dist/StatisticsChart';
import { fetchStatistics } from '@/services/swagger/KnowledgeGraph';
import { logoutWithRedirect, isAuthenticated } from '@/components/util';
import { Empty, Row } from 'antd';

import './index.less';

const Statistics: React.FC = () => {
    const [relationStat, setRelationStat] = useState<RelationStat[]>([]);
    const [entityStat, setEntityStat] = useState<EntityStat[]>([]);

    useEffect(() => {
        console.log("isAuthenticated in ModelConfig: ", isAuthenticated());
        if (!isAuthenticated()) {
            logoutWithRedirect();
        } else {
            fetchStatistics().then((data) => {
                console.log("fetchStatistics data: ", data);
                const relationStats = data.relation_stat;
                setRelationStat(relationStats);

                const entityStats = data.entity_stat;
                setEntityStat(entityStats);
            });
        }
    }, []);

    return (
        <Row className="statistics-container">{
            (relationStat && entityStat) ?
                <StatisticsChart nodeStat={entityStat} edgeStat={relationStat} />
                : <Empty />
        }</Row>
    );
};

export default Statistics;
