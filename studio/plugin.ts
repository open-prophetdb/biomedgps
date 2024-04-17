import { IApi } from 'umi'

export default (api: IApi) => {
    // For GTEx Components
    api.addHTMLLinks(() => {
        return [
            // We don't like the default bootstrap style which messes up our site.
            // {
            //     rel: "stylesheet",
            //     href: "https://gtexportal.org/external/bootstrap/3.3.7/bootstrap.min.css"
            // },
            // {
            //     rel: "stylesheet",
            //     href: "https://gtexportal.org/external/jquery-ui-1.11.4.custom/jquery-ui.css"
            // },
            // {
            //     rel: "stylesheet",
            //     href: "https://use.fontawesome.com/releases/v5.5.0/css/all.css"
            // }
        ]
    })

    // For GTEx Components
    api.addHTMLHeadScripts(() => {
        return [
            "https://drugs.3steps.cn/assets/js/jquery-1.11.2.min.js",
            "https://drugs.3steps.cn/assets/js/popper-1.11.0.min.js",
            "https://gtexportal.org/external/jquery-ui-1.11.4.custom/jquery-ui.min.js",
            "https://gtexportal.org/external/bootstrap/3.3.7/bootstrap.min.js"
        ]
    })
};