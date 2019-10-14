module.exports = {
  name: 'scrum-poker',
  preset: '../../jest.config.js',
  coverageDirectory: '../../coverage/apps/scrum-poker',
  snapshotSerializers: [
    'jest-preset-angular/AngularSnapshotSerializer.js',
    'jest-preset-angular/HTMLCommentSerializer.js'
  ]
};
