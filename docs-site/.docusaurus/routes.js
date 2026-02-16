import React from 'react';
import ComponentCreator from '@docusaurus/ComponentCreator';

export default [
  {
    path: '/cfdl/search',
    component: ComponentCreator('/cfdl/search', '216'),
    exact: true
  },
  {
    path: '/cfdl/',
    component: ComponentCreator('/cfdl/', '93d'),
    routes: [
      {
        path: '/cfdl/',
        component: ComponentCreator('/cfdl/', 'd26'),
        routes: [
          {
            path: '/cfdl/',
            component: ComponentCreator('/cfdl/', '414'),
            routes: [
              {
                path: '/cfdl/examples',
                component: ComponentCreator('/cfdl/examples', '9e1'),
                exact: true,
                sidebar: "tutorialSidebar"
              },
              {
                path: '/cfdl/examples/cre_developer',
                component: ComponentCreator('/cfdl/examples/cre_developer', '82b'),
                exact: true
              },
              {
                path: '/cfdl/examples/cre_development_with_financing',
                component: ComponentCreator('/cfdl/examples/cre_development_with_financing', '9b9'),
                exact: true
              },
              {
                path: '/cfdl/examples/cre_lease_up',
                component: ComponentCreator('/cfdl/examples/cre_lease_up', 'f3b'),
                exact: true
              },
              {
                path: '/cfdl/examples/cre_multi_file',
                component: ComponentCreator('/cfdl/examples/cre_multi_file', '655'),
                exact: true
              },
              {
                path: '/cfdl/examples/cre_phased',
                component: ComponentCreator('/cfdl/examples/cre_phased', 'fa5'),
                exact: true
              },
              {
                path: '/cfdl/examples/cre-examples',
                component: ComponentCreator('/cfdl/examples/cre-examples', '1e0'),
                exact: true,
                sidebar: "tutorialSidebar"
              },
              {
                path: '/cfdl/examples/first_stream',
                component: ComponentCreator('/cfdl/examples/first_stream', 'a12'),
                exact: true,
                sidebar: "tutorialSidebar"
              },
              {
                path: '/cfdl/examples/minimal_model',
                component: ComponentCreator('/cfdl/examples/minimal_model', 'b52'),
                exact: true,
                sidebar: "tutorialSidebar"
              },
              {
                path: '/cfdl/examples/multi_file',
                component: ComponentCreator('/cfdl/examples/multi_file', '1aa'),
                exact: true,
                sidebar: "tutorialSidebar"
              },
              {
                path: '/cfdl/examples/opco_basic',
                component: ComponentCreator('/cfdl/examples/opco_basic', 'f8f'),
                exact: true
              },
              {
                path: '/cfdl/examples/opco_multi_file',
                component: ComponentCreator('/cfdl/examples/opco_multi_file', '613'),
                exact: true
              },
              {
                path: '/cfdl/examples/opco_with_growth',
                component: ComponentCreator('/cfdl/examples/opco_with_growth', '219'),
                exact: true
              },
              {
                path: '/cfdl/examples/operating-business-examples',
                component: ComponentCreator('/cfdl/examples/operating-business-examples', 'cc8'),
                exact: true,
                sidebar: "tutorialSidebar"
              },
              {
                path: '/cfdl/examples/simple_contract',
                component: ComponentCreator('/cfdl/examples/simple_contract', '4f7'),
                exact: true,
                sidebar: "tutorialSidebar"
              },
              {
                path: '/cfdl/examples/with_pack',
                component: ComponentCreator('/cfdl/examples/with_pack', '824'),
                exact: true,
                sidebar: "tutorialSidebar"
              },
              {
                path: '/cfdl/getting-started',
                component: ComponentCreator('/cfdl/getting-started', 'ce6'),
                exact: true,
                sidebar: "tutorialSidebar"
              },
              {
                path: '/cfdl/install-configure',
                component: ComponentCreator('/cfdl/install-configure', 'e2c'),
                exact: true,
                sidebar: "tutorialSidebar"
              },
              {
                path: '/cfdl/language-guide',
                component: ComponentCreator('/cfdl/language-guide', '2e9'),
                exact: true,
                sidebar: "tutorialSidebar"
              },
              {
                path: '/cfdl/language-reference',
                component: ComponentCreator('/cfdl/language-reference', '5ef'),
                exact: true,
                sidebar: "tutorialSidebar"
              },
              {
                path: '/cfdl/language-reference/compiler-spec',
                component: ComponentCreator('/cfdl/language-reference/compiler-spec', 'a2f'),
                exact: true,
                sidebar: "tutorialSidebar"
              },
              {
                path: '/cfdl/language-reference/diagnostics',
                component: ComponentCreator('/cfdl/language-reference/diagnostics', 'f7f'),
                exact: true,
                sidebar: "tutorialSidebar"
              },
              {
                path: '/cfdl/language-reference/grammar',
                component: ComponentCreator('/cfdl/language-reference/grammar', '055'),
                exact: true,
                sidebar: "tutorialSidebar"
              },
              {
                path: '/cfdl/language-reference/language-spec',
                component: ComponentCreator('/cfdl/language-reference/language-spec', '22b'),
                exact: true,
                sidebar: "tutorialSidebar"
              },
              {
                path: '/cfdl/language-reference/pack-interface',
                component: ComponentCreator('/cfdl/language-reference/pack-interface', 'dc8'),
                exact: true,
                sidebar: "tutorialSidebar"
              },
              {
                path: '/cfdl/packs',
                component: ComponentCreator('/cfdl/packs', '3f4'),
                exact: true,
                sidebar: "tutorialSidebar"
              },
              {
                path: '/cfdl/troubleshooting',
                component: ComponentCreator('/cfdl/troubleshooting', 'dbf'),
                exact: true,
                sidebar: "tutorialSidebar"
              },
              {
                path: '/cfdl/',
                component: ComponentCreator('/cfdl/', 'c15'),
                exact: true,
                sidebar: "tutorialSidebar"
              }
            ]
          }
        ]
      }
    ]
  },
  {
    path: '*',
    component: ComponentCreator('*'),
  },
];
