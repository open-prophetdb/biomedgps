// @ts-nocheck
import React, { useEffect, useState } from 'react'
import {
  GTexTranscriptViewer, GTexGeneBoxplotViewer,
  GTexGeneViolinViewer
} from 'biominer-components';

import './index.less';

type GTexViewerProps = {
  rootId?: string,
  type: string, // gene or transcript
  title?: string,
  officialGeneSymbol: string // e.g. 'PRG4', only support human gene for now
}

const defaultSummary = "Overall, this table provides insights into the tissue-specific expression pattern of the PRG4 gene in human tissues, as well as the specific transcript variants that are expressed in each tissue. The median expression levels suggest that PRG4 is highly expressed in some tissues, such as Adipose_Visceral_Omentum and Artery_Tibial, but not expressed or expressed at very low levels in other tissues, such as Bladder and Brain_Amygdala. The information in this table can be used to gain a better understanding of the role of PRG4 in different tissues and may be useful in designing future studies investigating the gene's function in health and disease."

const GTexViewer: React.FC<GTexViewerProps> = (props) => {
  const [rootId, setRootId] = useState<string>("");
  const [summary, setSummary] = useState<string>(defaultSummary);

  useEffect(() => {
    if (!props.rootId) {
      setRootId('gtex-viewer')
    } else {
      setRootId(props.rootId)
    }
  }, []);

  return (
    <div className='gtex-viewer'>
      <div className='summary'>
        <h3 className='summary-title'>Summary [Summarized by AI]</h3>
        <p className='summary-content'>
          {summary}
        </p>
      </div>
      {
        props.type == 'transcript' ?
          <div className='transcript-figures'>
            <GTexTranscriptViewer
              rootId={rootId + '-isoform-transposed'}
              type="isoformTransposed" title={props.title || "Isoform Transposed"}
              geneId={props.officialGeneSymbol} />
            <GTexTranscriptViewer
              rootId={rootId + '-exon'} title={props.title || "Exon"}
              type="exon" geneId={props.officialGeneSymbol} />
            <GTexTranscriptViewer
              rootId={rootId + '-junction'} title={props.title || "Junction"}
              type="junction" geneId={props.officialGeneSymbol} />
          </div>
          : null
      }
      {
        props.type == 'gene' ?
          <div className='gene-figures'>
            <GTexGeneBoxplotViewer rootId={rootId + 'boxplot'}
              title={props.title || 'Boxplot'}
              geneId={props.officialGeneSymbol} />
            <GTexGeneViolinViewer rootId={rootId + 'violin'}
              title={props.title || 'Violin Plot'}
              geneId={props.officialGeneSymbol} />
          </div>
          : null
      }
    </div>
  )
}

export default GTexViewer;