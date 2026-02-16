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
    component: ComponentCreator('/cfdl/', '340'),
    routes: [
      {
        path: '/cfdl/',
        component: ComponentCreator('/cfdl/', 'f4e'),
        routes: [
          {
            path: '/cfdl/',
            component: ComponentCreator('/cfdl/', '224'),
            routes: [
              {
                path: '/cfdl/getting-started',
                component: ComponentCreator('/cfdl/getting-started', '8dd'),
                exact: true,
                sidebar: "tutorialSidebar"
              },
              {
                path: '/cfdl/install-configure',
                component: ComponentCreator('/cfdl/install-configure', 'fd2'),
                exact: true,
                sidebar: "tutorialSidebar"
              },
              {
                path: '/cfdl/language-guide',
                component: ComponentCreator('/cfdl/language-guide', '940'),
                exact: true,
                sidebar: "tutorialSidebar"
              },
              {
                path: '/cfdl/packs',
                component: ComponentCreator('/cfdl/packs', 'c19'),
                exact: true,
                sidebar: "tutorialSidebar"
              },
              {
                path: '/cfdl/reference',
                component: ComponentCreator('/cfdl/reference', '259'),
                exact: true,
                sidebar: "tutorialSidebar"
              },
              {
                path: '/cfdl/troubleshooting',
                component: ComponentCreator('/cfdl/troubleshooting', '087'),
                exact: true,
                sidebar: "tutorialSidebar"
              },
              {
                path: '/cfdl/',
                component: ComponentCreator('/cfdl/', 'abe'),
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
