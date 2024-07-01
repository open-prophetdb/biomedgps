import React, { useRef, useEffect } from 'react';
// @ts-ignore
import { GeneExpressionViolinPlot } from 'gtex-d3';
// @ts-ignore
import jquery from 'jquery';
// @ts-ignore
import { parseGenes, getGtexUrls } from 'gtex-d3/src/modules/gtexDataParser';
import 'gtex-d3/css/geneExpViolin.css';

import './index.less';

type GeneViewerProps = {
  /**
   * @description Only support Hugo gene symbol or Ensembl gene ID. e.g. "TP53" or "ENSG00000141510"
   */
  geneId: string;
  /**
   * @description Title of the plot.
   */
  title?: string;
};

const host =
  'https://gtexportal.org/api/v2/reference/geneSearch?geneId=';

const GTexGeneViolinViewer: React.FC<GeneViewerProps> = (props) => {
  const ref = useRef(null);

  const { geneId, title } = props;

  const removeChildren = (tag: HTMLElement) => {
    if (tag.children) {
      tag.innerHTML = '';
    }
  };

  const fetchGene = async (geneId: string) => {
    const response = await fetch(`${host}${geneId}`);
    const data = await response.json();
    return data;
  };

  const launch = (element: HTMLElement, gencodeId: string) => {
    // @ts-ignore
    window.$ = jquery;

    console.log("Launch violin plot: ", GeneExpressionViolinPlot);
    const width = element.clientWidth - 100;
    // (Re)render the plot
    GeneExpressionViolinPlot.launchBulkTissueViolinPlot(
      "gene-expr-vplot",
      "gene-expr-vplot-tooltip",
      gencodeId,
      '', // title
      getGtexUrls(), // urls
      {
        top: 50,
        right: 75,
        bottom: 150,
        left: 60,
      }, // margins
      {
        w: width,
        h: 250,
      }, // dimensions
    );
  }

  const update = () => {
    // Remove existing children from the container element
    if (ref.current) {
      const element = ref.current as HTMLElement;
      removeChildren(element);

      fetchGene(geneId)
        .then((data) => {
          const gene = parseGenes(data, true, geneId);
          const gencodeId = gene.gencodeId;

          launch(element, gencodeId);
        })
        .catch((error) => {
          console.log(error);
        });
    }
  };

  useEffect(() => {
    if (ref.current && geneId) {
      if (/.*\.[0-9]+/.test(geneId)) {
        launch(ref.current as HTMLElement, geneId);
      } else {
        update();
      }
    }
  }, [geneId]);

  return (
    <div className="gtex-gene-violin-viewer">
      {title && <h3>{title}</h3>}
      <div id="gene-expr-vplot" style={{ width: '100%' }} ref={ref} />
      <div className="modal fade" id="gene-expr-vplot-filter-modal" tabIndex={-1} role="dialog" aria-labelledby="Gene Expr Violin Plot Tissue Filter Modal" aria-hidden="true">
        <div className="modal-dialog modal-lg" role="document">
          <div className="modal-content">
            <div className="modal-header">
              <h5 className="modal-title">Tissue Filter</h5>
              <button id="gene-expr-vplot-filter-modal-close" type="button" data-dismiss="modal"
                aria-label="Close" className="close">
                <span aria-hidden="true"><i className="far fa-times-circle"></i></span>
              </button>
            </div>
            <div className="modal-body">
              <div className="row" id="gene-expr-vplot-filter-modal-body"></div>
            </div>
            <div className="modal-footer">
              <button id="gene-expr-vplot-filter-modal-button" type="button"
                data-dismiss="modal" className="btn btm-sm btn-default"
                style={{ fontSize: '0.9rem' }}>
                Apply
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
};

export default GTexGeneViolinViewer;
