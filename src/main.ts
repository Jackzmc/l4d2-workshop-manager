import { createApp } from "vue";
import App from "./App.vue";

import Buefy from 'buefy';
import 'buefy/dist/buefy.css';

import { library } from '@fortawesome/fontawesome-svg-core'
import { faAngleDown, faAngleUp, faCog, faList, faWindowMinimize, faTimes, faExclamationTriangle, faPlus, faSearch, faLink, faArrowUp, faAngleRight, faLongArrowAltLeft, faSync } from '@fortawesome/free-solid-svg-icons'
import { faWindowMaximize } from '@fortawesome/free-regular-svg-icons'
import { FontAwesomeIcon } from '@fortawesome/vue-fontawesome'

library.add( faAngleDown, faAngleUp, faCog, faList, faWindowMinimize, faWindowMaximize, faTimes, faExclamationTriangle, faPlus, faSearch, faLink, faAngleRight, faArrowUp, faLongArrowAltLeft, faSync )


createApp( App )
    .use( Buefy, {
        defaultIconPack: 'fas',
        defaultIconComponent: "font-awesome-icon",
        defaultSnackbarDuration: 5000
    } )
    .component( 'font-awesome-icon', FontAwesomeIcon )
    .mount( '#app' )
