import React from 'react';
import TemplatePage from '@site/src/components/TemplatePage';
import { templates } from '@site/src/data/templates';

export default function Page() {
  const template = templates.find(t => t.id === 'underwater-world');
  if (!template) throw new Error('Template not found');
  return <TemplatePage template={template} />;
}
