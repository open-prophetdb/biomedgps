import React, { useRef, useEffect } from 'react';
// @ts-ignore
import { TranscriptBrowser } from 'gtex-d3';
import './index.less';

type TranscriptViewerProps = {
  /**
   * @description The id of the container element, if you want to render multiple plots in the same page, you need to specify different rootId for each plot.
   */
  rootId: string;
  /**
   * @description Only support Hugo gene symbol or Ensembl gene ID. e.g. "TP53" or "ENSG00000141510"
   */
  geneId: string;
  /**
   * @description The title of the plot
   */
  title?: string;
  /**
   * @description The type of the plot
   */
  type: 'exon' | 'junction' | 'isoformTransposed';
};

const GTexTranscriptViewer: React.FC<TranscriptViewerProps> = (props) => {
  const ref = useRef(null);

  const { rootId, type, geneId, title } = props;

  const removeChildren = (tag: HTMLElement) => {
    if (tag.children) {
      tag.innerHTML = '';
    }
  };

  const update = () => {
    // Remove existing children from the container element
    if (ref.current) {
      removeChildren(ref.current as HTMLElement);
    }

    // (Re)render the plot
    TranscriptBrowser.render(type, geneId, rootId);
  };

  useEffect(() => {
    if (ref.current && geneId && type) {
      update();
    }
  }, [rootId, type, geneId]);

  return (
    <div className="gtex-transcript-viewer">
      {title && <h3>{title}</h3>}
      <div id={rootId} style={{ width: '100%' }} ref={ref} />
      {/* <div id={`${rootId}-isoformToolbar`}></div> */}
      {/* <div id={`${rootId}-isoformClone`} ></div> */}
    </div>
  );
};

export default GTexTranscriptViewer;
